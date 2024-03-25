use std::{ffi::OsStr, fs, path::{Path, PathBuf}};
use regex::Regex;

pub fn resolve(path: &Path) -> Option<Vec<PathBuf>> {
    let mut fin: Vec<PathBuf> = Vec::new();

    match path.iter().nth(0) {
        Some(e) => {
            let mut t = PathBuf::new();
            t.push(e);
            _ = t.canonicalize();
            fin.push(t)
        },
        None => { return Some(fin); }
    }

    for pe in path.iter().skip(1) {
        if fin.is_empty() { return Some(fin); }

        fin = get_next_file_layer(fin, pe);
    }
    
    Some(fin)
}

fn get_next_file_layer(current_layer: Vec<PathBuf>, next_element: &OsStr) -> Vec<PathBuf> {
    let mut new_layer: Vec<PathBuf> = Vec::new();

    for p in &current_layer {
        if p.is_file() { continue; }
        
        let mut candidates = match next_element.to_str().unwrap().contains("*") {
            false => {
                if p.join(next_element).try_exists().unwrap() {
                    vec![PathBuf::from(next_element)]
                } else {
                    vec![]
                }
            },
            true => {
                let re = Regex::new(
                    &format!("^{}$", &next_element.to_str().unwrap().replace(".", "[.]").replace("*", ".*"))
                ).unwrap();

                let regex_filter = |x: PathBuf| -> Option<PathBuf> {
                    if re.is_match(x.iter().last().unwrap().to_str().unwrap()) {
                        return Some(p.join(x))
                    } else {
                        return None
                    };
                };

                fs::read_dir(p).unwrap()  // FIX WHEN SOME ENTRIES ARE NOT DIRECTORIES
                    .map(|x| PathBuf::from(x.unwrap().file_name()))
                    .filter_map(regex_filter)
                    .collect()
            }
        };

        candidates = candidates.into_iter()
            .filter_map(|x|
                if x.is_symlink() {
                    Some(x.read_link().unwrap())
                }
                else {
                    Some(x)
                }
            )
            .map(|x| p.join(x))
            .collect();

        new_layer.append(&mut candidates);
    }
    
    new_layer
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::{Alphanumeric, DistString};
    use std::fs;

    fn test_setup() -> PathBuf {
        let test_dir = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        let mut tdr = std::env::temp_dir(); // THIS DOESN'T DELETE AFTER RUNNING????
        tdr.push(test_dir);
        _ = fs::create_dir(&tdr);

        tdr
    }

    fn validate(mut output: Vec<PathBuf>, mut solution: Vec<PathBuf>, test_dir: Option<PathBuf>) {
        match test_dir {
            Some(p) => { _ = fs::remove_dir_all(p); }
            None => ()
        }

        output.sort_by_key( |k| k.as_os_str().to_owned());
        solution.sort_by_key( |k| k.as_os_str().to_owned());

        dbg!(&output);
        dbg!(&solution);

        assert_eq!(output, solution);
    }

    #[test]
    fn ending_asterisk() {
        let tdr = test_setup();
        let test_path = tdr.clone().join("*");

        let solution: Vec<PathBuf> = vec![
            tdr.join("A"),
            tdr.join("B"),
        ];

        for p in &solution {
            _ = fs::create_dir(p);
        }
        
        validate(
            resolve(&test_path).unwrap(),
            solution,
            Some(tdr)
        );
    }

    #[test]
    fn dot_path() {
        let tdr = test_setup();
        let test_path = tdr.clone().join("*.*").join("*");

        _ = fs::create_dir(tdr.join("A.B"));
        _ = fs::create_dir(tdr.join("A.B").join("C"));

        let solution: Vec<PathBuf> = vec![
            tdr.join("A.B").join("C")
        ];

        validate(
            resolve(&test_path).unwrap(),
            solution,
            Some(tdr)
        )
    }

    #[test]
    fn double_asterisk() {
        let tdr = test_setup();
        let test_input = tdr.clone().join("*").join("*");

        _ = fs::create_dir(tdr.join("A"));
        _ = fs::create_dir(tdr.join("B"));

        let solution: Vec<PathBuf> = vec![
            tdr.join("A").join("C"),
            tdr.join("B").join("D"),
        ];

        for p in &solution {
            _ = fs::create_dir(p);
        }
        
        validate(
            resolve(&test_input).unwrap(),
            solution,
            Some(tdr)
        );
    }

    #[test]
    fn not_all_items() {
        let tdr = test_setup();
        let test_input = tdr.clone().join("X*");

        let mut paths: Vec<PathBuf> = vec![
            tdr.join("XA"),
            tdr.join("X")
        ];

        let solution = paths.clone();

        paths.push(tdr.join("YB"));
        paths.push(tdr.join("YX"));
        for p in paths {
            _ = fs::create_dir(p);
        }

        validate(
            resolve(&test_input).unwrap(),
            solution,
            Some(tdr)
        );
    }

    #[test]
    fn double_compound() {
        let tdr = test_setup();
        let test_input = tdr.clone().join("X*").join("Y").join("*Z");

        let first_layer: Vec<PathBuf> = vec![
            tdr.join("X"),
            tdr.join("XA"),
            tdr.join("YB"),
            tdr.join("YX"),
        ];
        for p in &first_layer { _ = fs::create_dir(p); }

        let second_layer: Vec<PathBuf> = vec![
            tdr.join("X").join("Y"),
            tdr.join("X").join("TY"),
            tdr.join("XA").join("Y"),
            tdr.join("YB").join("Y")
        ];
        for p in &second_layer { _ = fs::create_dir(p); }

        let mut third_layer: Vec<PathBuf> = vec![
            tdr.join("X").join("Y").join("Z"),
            tdr.join("XA").join("Y").join("Z"),
            tdr.join("XA").join("Y").join("TZ"),
            tdr.join("XA").join("Y").join("ZAZ"),
            tdr.join("XA").join("Y").join("ZZZ"),
        ];

        let solution = third_layer.clone();

        third_layer.push(tdr.join("X").join("Y").join("z"));
        third_layer.push(tdr.join("X").join("TY").join("Z"));
        third_layer.push(tdr.join("XA").join("Y").join("ZA"));

        for p in &third_layer { _ = fs::create_dir(p); }

        validate(
            resolve(&test_input).unwrap(),
            solution,
            Some(tdr)
        );
    }

    #[test]
    fn files() {
        let tdr = test_setup();
        let test_path = tdr.clone().join("A").join("*.jp*g");

        _ = fs::create_dir(tdr.join("A"));
        _ = fs::File::create(tdr.join("A").join("a.jpg"));
        _ = fs::File::create(tdr.join("A").join("b.jpeg"));
        _ = fs::File::create(tdr.join("A").join("c.jg"));

        let solution: Vec<PathBuf> = vec![
            tdr.join("A").join("a.jpg"),
            tdr.join("A").join("b.jpeg")
        ];

        validate(
            resolve(&test_path).unwrap(),
            solution,
            Some(tdr)
        );
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn symlinks_unix() {
        let tdr = test_setup();
        let test_path = tdr.clone().join("A").join("*").join("*");

        _ = fs::create_dir(tdr.join("A"));
        _ = fs::create_dir(tdr.join("B"));
        _ = fs::create_dir(tdr.join("B").join("C"));

        _ = std::os::unix::fs::symlink(
            tdr.join("B"),
            tdr.join("A").join("X")
        );

        let solution: Vec<PathBuf> = vec![tdr.join("B").join("C")];

        validate(
            resolve(&test_path).unwrap(),
            solution,
            Some(tdr)
        );
    }

    #[test]
    fn doc_example() {
        let tdr = test_setup();

        _ = fs::create_dir(tdr.join("blogs"));

        _ = fs::create_dir(tdr.join("blogs").join("blog_1"));
        _ = fs::create_dir(tdr.join("blogs").join("blog_1").join("assets"));
        _ = fs::File::create(tdr.join("blogs").join("blog_1").join("post.txt"));
        _ = fs::File::create(tdr.join("blogs").join("blog_1").join("assets").join("logo.jpeg"));

        _ = fs::create_dir(tdr.join("blogs").join("blog_2"));
        _ = fs::create_dir(tdr.join("blogs").join("blog_2").join("assets"));
        _ = fs::File::create(tdr.join("blogs").join("blog_2").join("post.txt"));
        _ = fs::File::create(tdr.join("blogs").join("blog_2").join("assets").join("research_notes.txt"));
        
        _ = fs::create_dir(tdr.join("videos"));
        
        _ = fs::create_dir(tdr.join("videos").join("video_1"));
        _ = fs::create_dir(tdr.join("videos").join("video_1").join("assets"));
        _ = fs::File::create(tdr.join("videos").join("video_1").join("script.txt"));
        _ = fs::File::create(tdr.join("videos").join("video_1").join("assets").join("logo.jpeg"));

        _ = fs::create_dir(tdr.join("videos").join("video_2"));
        _ = fs::create_dir(tdr.join("videos").join("video_2").join("assets"));
        _ = fs::File::create(tdr.join("videos").join("video_2").join("script.txt"));
        _ = fs::File::create(tdr.join("videos").join("video_2").join("assets").join("sound_effect.wav"));

        _ = fs::create_dir(tdr.join("videos").join("video_3"));
        _ = fs::create_dir(tdr.join("videos").join("video_3").join("assets"));
        _ = fs::File::create(tdr.join("videos").join("video_3").join("script.txt"));
        _ = fs::File::create(tdr.join("videos").join("video_3").join("assets").join("new_logo.png"));


        let mut test_path = tdr.clone().join("*").join("*").join("*.txt");
        let mut solution: Vec<PathBuf> = vec![
            tdr.join("blogs").join("blog_1").join("post.txt"),
            tdr.join("blogs").join("blog_2").join("post.txt"),
            tdr.join("videos").join("video_1").join("script.txt"),
            tdr.join("videos").join("video_2").join("script.txt"),
            tdr.join("videos").join("video_3").join("script.txt")
        ];

        validate(
            resolve(&test_path).unwrap(),
            solution,
            None
        );

        test_path = tdr.clone().join("*").join("*").join("assets").join("*logo*");
        solution = vec![
            tdr.join("blogs").join("blog_1").join("assets").join("logo.jpeg"),
            tdr.join("videos").join("video_1").join("assets").join("logo.jpeg"),
            tdr.join("videos").join("video_3").join("assets").join("new_logo.png")
        ];

        validate(
            resolve(&test_path).unwrap(),
            solution,
            None
        );

        test_path = tdr.clone().join("*").join("*_1").join("assets").join("*logo*");
        solution = vec![
            tdr.join("blogs").join("blog_1").join("assets").join("logo.jpeg"),
            tdr.join("videos").join("video_1").join("assets").join("logo.jpeg")
        ];

        validate(
            resolve(&test_path).unwrap(),
            solution,
            None
        );

        _ = fs::create_dir(tdr.join("presentations"));
        _ = fs::create_dir(tdr.join("presentations").join("presentation_1"));
        _ = fs::create_dir(tdr.join("presentations").join("presentation_1").join("assets"));
        _ = fs::File::create(tdr.join("presentations").join("presentation_1").join("assets").join("logo.jpeg"));

        solution = vec![
            tdr.join("blogs").join("blog_1").join("assets").join("logo.jpeg"),
            tdr.join("videos").join("video_1").join("assets").join("logo.jpeg"),
            tdr.join("presentations").join("presentation_1").join("assets").join("logo.jpeg")
        ];

        validate(
            resolve(&test_path).unwrap(),
            solution,
            Some(tdr)
        );
    }
}