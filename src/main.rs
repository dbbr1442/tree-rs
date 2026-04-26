#![allow(unused)]

use core::slice;
use std::{path::{Path, PathBuf}, str::FromStr};

use lesbian_parser::{ArgumentData, Parse};

use crate::color::{Color256, GetEscape};

mod color;

#[derive(Debug)]
struct Options {
    full_paths: bool,
    absolute_paths: bool,
    depth: u32,
    root_path: PathBuf,
    follow_links: bool,
}

impl Parse for Options {
    fn default_args() -> Self {
        Self { full_paths: false, absolute_paths: false, root_path: PathBuf::from_str(".").unwrap(), depth: u32::MAX, follow_links: false }
    }
    fn add_flags<'a>(&'a mut self, parser: &mut lesbian_parser::ParserData<'a>) {
        parser.add_arg(ArgumentData::new_with_short("--full-paths", "-f", false, &mut self.full_paths));
        parser.add_arg(ArgumentData::new_with_short("--absolute-paths", "-a", false, &mut self.absolute_paths));
        parser.add_arg(ArgumentData::new_with_short("--depth", "-d", false, &mut self.depth));
        parser.add_arg(ArgumentData::new_with_short("--path", "-p", false, &mut self.root_path));
        parser.add_arg(ArgumentData::new_with_short("--follow-links", "-fl", false, &mut self.follow_links));
    }
}

fn main() {
    let options = match Options::from_env_args() {
        Ok(val) => val,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    println!("{:?}", options);

    let depth_budget = if options.depth == u32::MAX {
        None
    } else {
        Some(options.depth)
    };

    let root_node = Node::from_path(&options.root_path, depth_budget, &options).unwrap();
    root_node.print_root(&options);
}

#[derive(Debug)]
enum Node {
    Directory {
        path: PathBuf,
        children: Vec<Node>,
    },
    File {
        path: PathBuf,
    },
    FollowedLink {
        path: PathBuf,
        points_to: Result<Box<Node>, String>,
    },
    Error {
        path: Option<PathBuf>,
        error: std::io::Error,
    },
    UnfollowedLink {
        path: PathBuf,
        points_to: Result<PathBuf, String>,
    }
}

impl Node {
    fn from_path(path: &Path, depth_budget: Option<u32>, options: &Options) -> Option<Self> {
        if !path.exists() {
            return None;
        }

        if path.is_file() {
            return Some(Self::File { path: path.to_path_buf() })
        }

        if path.is_symlink() && !options.follow_links {
            let points_to = path.read_link().map_err(|e|e.to_string());
            return Some(Self::UnfollowedLink { path: path.to_path_buf(), points_to })
        } else if path.is_symlink() {
            let points_to = path.read_link().map_err(|e|e.to_string());
            match points_to {
                Err(e) => return Some(Self::FollowedLink { path: path.to_path_buf(), points_to: Err(e) }),
                Ok(val) => {
                    let node = Self::from_path(&val, depth_budget.map(|v|v-1), options).unwrap();
                    return Some(Self::FollowedLink { path: path.to_path_buf(), points_to: Ok(Box::new(node)) });
                },
            }
        }

        if path.is_dir() {
            let files = path.read_dir();
            let files = files.map(|iter| {
                let mut children = Vec::new();

                'query_files: for file in iter {
                    let node = match file {
                        Err(e) => {
                            children.push(Self::Error { path: None, error: e });
                            continue 'query_files;
                        },
                        Ok(val) => {
                            let path = val.path();
                            
                            if let Some(val) = depth_budget {
                                if val == 0 { continue 'query_files; }
                            }
                            let node = Self::from_path(&path, depth_budget.map(|v|v-1), options).unwrap();
                            node
                        },
                    };

                    children.push(node);
                }

                children
            });

            let res = match files {
                Ok(val) => Self::Directory { path: path.to_path_buf(), children: val },
                Err(e) => Self::Error { path: Some(path.to_path_buf()), error: e },
            };
            
            return Some(res);
        }

        None

    }

    fn print_root(&self, options: &Options) {
        let dir = Color256::new(4).get_fg();
        let error = Color256::new(1).get_fg();
        let normal = Color256::NORMAL.get_fg();


        match self {
            Self::File { path } => println!("{}", path.display()),

            Self::UnfollowedLink { path, points_to } => {
                todo!()
            },

            Self::FollowedLink { path, points_to } => {
                todo!()
            },

            Self::Error { path, error } => {
                match path {
                    Some(val) => println!("{error}{} - {}{normal}", val.display(), error),
                    None => println!("{error}{}{normal}", error),
                }
            },

            Self::Directory { path, children } => {
                let path = if options.absolute_paths { &path.canonicalize().unwrap() } else { path };
                let path = if options.full_paths || options.absolute_paths { path.as_os_str() } else { 
                    match path.file_name() {
                        Some(val) => val,
                        None => path.as_os_str(),
                    }
                };
                println!("{dir}{}{normal}", path.display());
                Self::print_dir_recursive(children.as_slice(), 0, options, Vec::new());
            },
        }
    }

    fn print_dir_recursive(children: &[Self], depth_indentation: u32, options: &Options, mut exclude_n_depths: Vec<u32>) {

        let mid_line = (0..=depth_indentation).map(|v| {
            if v >= depth_indentation {
                "├── "
            } else {
                if exclude_n_depths.contains(&v) {
                    "    "
                } else {
                    "│   "
                }
            }
        }).collect::<String>();

        let end_line = (0..=depth_indentation).map(|v| {
            if v >= depth_indentation {
                "└── "
            } else {
                if exclude_n_depths.contains(&v) {
                    "    "
                } else {
                    "│   "
                }
            }
        }).collect::<String>();

        let len = children.len().checked_sub(1);

        let len = match len {
            None => return,
            Some(val) => val,
        };

        for (i, node) in children.iter().enumerate() {
            let at_end = i >= len;
            let prefix = if at_end {
                &end_line
            } else {
                &mid_line
            };

            let link = Color256::new(6).get_fg();
            let error = Color256::new(1).get_fg();
            let normal = Color256::NORMAL.get_fg();
            let dir = Color256::new(4).get_fg();
            
            match node {
                Self::File { path } => {
                    let path = if options.absolute_paths { &path.canonicalize().unwrap() } else { path };
                    let path = if options.full_paths || options.absolute_paths { path.as_os_str() } else { path.file_name().unwrap() };
                    println!("{}{}", prefix, path.display());
                },

                Self::UnfollowedLink { path, points_to } => {
                    let path = if options.absolute_paths { &path.canonicalize().unwrap() } else { path };
                    let path = if options.full_paths || options.absolute_paths { path.as_os_str() } else { path.file_name().unwrap() };

                    match points_to {
                        Ok(val) => {
                            let color = get_path_color(&val).unwrap().get_fg();
                            println!("{prefix}{link}{}{normal} -> {color}{}{normal}", path.display(), val.display())
                        },
                        Err(e) => print!("{prefix}{link}{} -> {error}{}{normal}", path.display(), e),
                    }
                },

                Self::Directory { path, children } => {
                    let path = if options.absolute_paths { &path.canonicalize().unwrap() } else { path };
                    let path = if options.full_paths || options.absolute_paths { path.as_os_str() } else { path.file_name().unwrap() };
                    println!("{}{dir}{}{normal}", prefix, path.display());
                    if at_end { exclude_n_depths.push(depth_indentation); };
                    Self::print_dir_recursive(children.as_slice(), depth_indentation+1, options, exclude_n_depths.clone());
                },

                Self::Error { path, error } => {
                    match path {
                        Some(val) => println!("{prefix}{error}{} - {}{normal}", val.display(), error),
                        None => println!("{prefix}{error}{}{normal}", error),
                    }
                },

                Self::FollowedLink { path, points_to } => {
                    let path = if options.absolute_paths { &path.canonicalize().unwrap() } else { path };
                    let path = if options.full_paths || options.absolute_paths { path.as_os_str() } else { path.file_name().unwrap() };

                    match points_to {
                        Ok(val) => {
                            let color = val.get_type_color().get_fg();
                            print!("{prefix}{link}{}{normal} -> {color}", path.display());
                            let path = match val.get_path_or_err() {
                                Ok(path) => {
                                    let path = if options.absolute_paths { path.canonicalize().unwrap() } else { path };
                                    let path = if options.full_paths || options.absolute_paths { path.as_os_str() } else { path.file_name().unwrap() };
                                    println!("{}{normal}", path.display());

                                    if at_end { exclude_n_depths.push(depth_indentation); };
                                    match **val {
                                        Self::Directory { ref path, ref children } => 
                                            Self::print_dir_recursive(children.as_slice(), depth_indentation+1, options, exclude_n_depths.clone()),
                                        _ => {
                                            let slice = slice::from_ref(val.as_ref());
                                            Self::print_dir_recursive(slice, depth_indentation+1, options, exclude_n_depths.clone())
                                        },
                                    }
                                },
                                Err(e) => {
                                    let (path, err) = e.assume_err();
                                },
                            };
                        },
                        Err(e) => print!("{prefix}{link}{}{normal} -> {error}{}{normal}", path.display(), e),
                    }
                },
            }
        }
    }

    fn is_dir(&self) -> bool {
        if let Self::Directory {..} = self {
            true
        } else {
            false
        }
    }

    fn get_type_color(&self) -> Color256 {
        match self {
            Self::FollowedLink {..} => Color256::new(6),
            Self::UnfollowedLink {..} => Color256::new(6),
            Self::File {..} => Color256::NORMAL,
            Self::Error {..} => Color256::new(1),
            Self::Directory {..} => Color256::new(4),
        }
    }

    fn get_path_or_err(&self) -> Result<PathBuf, &Self> {
        match self {
            Self::UnfollowedLink { path, .. } => Ok(path.clone()),
            Self::FollowedLink { path, .. } => Ok(path.clone()),
            Self::File { path, .. } => Ok(path.clone()),
            Self::Directory { path, .. } => Ok(path.clone()),
            Self::Error { .. } => Err(self),
        }
    }
    
    fn assume_err(&self) -> (Option<&Path>, &std::io::Error) {
        match self {
            Self::Error { path, error } => {
                let path = path.as_ref().map(|v|v.as_path());
                return (path, error);
            },
            _ => panic!("Not error"),
        }
    }
}

fn get_path_color(path: &Path) -> Option<Color256> {
    if path.is_file() { return Some(Color256::NORMAL); }
    if path.is_dir() { return Some(Color256::new(4)); }
    if path.is_symlink() { return Some(Color256::new(6)); }
    None
}
