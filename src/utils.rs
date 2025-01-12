use std::path::{Path};
use std::sync::{Mutex, Arc};

pub fn get_partition_index<T>(vec : &Vec<T>, num_of_divisions : i32) -> Vec<usize>{
    let mut index_vec = Vec::new();
    let num_per_slice = vec.len()/num_of_divisions as usize;
    index_vec.push(0);
    for i in 0..num_of_divisions -1 {
        index_vec.push(num_per_slice*i as usize);
    }
    return index_vec;
}

pub fn print_lines(vec : &Vec<String>) {
    for line in vec{
        println!("{}", line);
    }
}

pub fn return_matches<T: AsRef<Path>, S : AsRef<str>>(path_list : &Vec<T>, string : &S ) -> Vec<String> {
    let mut matches : Vec<String> = Vec::new();
    //for each path in path list, tries to convert it to string and push to matches list if it contains string
    for p in path_list {
        (|| -> Option<()> {
            let file_name = p.as_ref().file_name()?.to_str()?;
            if file_name.to_lowercase().contains(&string.as_ref().to_lowercase()){
                matches.push(String::from(p.as_ref().to_str()?));
            }
            Some(())
        })();
    }
    return matches;
} 
//safe functions get lock before attempting to use fuc
pub fn safe_is_empty<T>(arc_mutex_vec : &Arc<Mutex<Vec<T>>>) -> bool {
    //gets access
    let vec = arc_mutex_vec.lock().unwrap();
    return vec.is_empty();
    //releases lock as soon as scope ends
}