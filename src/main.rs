use std::path::{Path, PathBuf};

use crate::color::{Color256, GetEscape};

mod color;

struct Options {
    full_paths: bool,
}

fn main() {
    let mut args = std::env::args().collect::<Vec<String>>();
    let path = args.get(1).expect("First argument must be filepath");
    let depth = args.get(2).map(|f|f.parse::<u32>().expect("Depth must be a number."));

    let mut root_node = Node::from_path(&Path::new(&path), depth).unwrap();
    //root_node.sort_dir_first_recursive();
    root_node.print_root(&Options { full_paths: false });
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
    Link {
        path: PathBuf,
        points_to: Result<PathBuf, String>,
    },
    Error {
        path: Option<PathBuf>,
        error: std::io::Error,
    },
}

impl Node {
    fn from_path(path: &Path, depth_budget: Option<u32>) -> Option<Self> {
        if !path.exists() {
            return None;
        }

        if path.is_file() {
            return Some(Self::File { path: path.to_path_buf() })
        }

        if path.is_symlink() {
            let points_to = path.read_link().map_err(|e|e.to_string());
            return Some(Self::Link { path: path.to_path_buf(), points_to })
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
                            let node = Self::from_path(&path, depth_budget.map(|v|v-1)).unwrap();
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

    fn sort_dir_first_recursive(&mut self) {
        if let Self::Directory { path, children } = self {
            children.sort_by(|a, b| {
                if a.is_dir() && b.is_dir() {
                    std::cmp::Ordering::Equal
                } else if a.is_dir() && !b.is_dir() {
                    std::cmp::Ordering::Greater
                } else if b.is_dir() && !a.is_dir() {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            });

            for entry in children.iter_mut() {
                if entry.is_dir() { entry.sort_dir_first_recursive() };
            } 
        }
    }

    fn print_root(&self, options: &Options) {
        let blue = Color256::new(4).get_fg();
        let error = Color256::new(1).get_fg();
        let normal = Color256::NORMAL.get_fg();


        match self {
            Self::File { path } => println!("{}", path.display()),
            Self::Link { path, points_to } => {
                todo!()
            },
            Self::Error { path, error } => {
                match path {
                    Some(val) => println!("{error}{} - {}{normal}", val.display(), error),
                    None => println!("{error}{}{normal}", error),
                }
            },

            Self::Directory { path, children } => {
                println!("{blue}{}{normal}", path.display());
                let near_end = children.len() <= 1;
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
            
            match node {
                Self::File { path } => {
                    let path = if options.full_paths { path.as_os_str() } else { path.file_name().unwrap() };
                    println!("{}{}", prefix, path.display());
                },
                Self::Link { path, points_to } => {
                    let red = Color256::new(6).get_fg();
                    let error = Color256::new(1).get_fg();
                    let normal = Color256::NORMAL.get_fg();

                    match points_to {
                        Ok(val) => println!("{prefix}{red}{} -> {}{normal}", path.display(), val.display()),
                        Err(e) => print!("{prefix}{red}{} -> {error}{}{normal}", path.display(), e),
                    }
                },
                Self::Directory { path, children } => {
                    let blue = Color256::new(4).get_fg();
                    let normal = Color256::NORMAL.get_fg();
                    let path = if options.full_paths { path.as_os_str() } else { path.file_name().unwrap() };
                    println!("{}{blue}{}{normal}", prefix, path.display());
                    if at_end { exclude_n_depths.push(depth_indentation); };
                    Self::print_dir_recursive(children.as_slice(), depth_indentation+1, options, exclude_n_depths.clone());
                }
                Self::Error { path, error } => {
                    let error_c = Color256::new(1).get_fg();
                    let normal = Color256::NORMAL.get_fg();
                    match path {
                        Some(val) => println!("{prefix}{error_c}{} - {}{normal}", val.display(), error),
                        None => println!("{prefix}{error_c}{}{normal}", error),
                    }
                }
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
}
