use core::slice;
use std::collections::VecDeque;
use std::error::Error;
use std::fs::{self, read_dir, DirEntry, ReadDir};
use std::io::Error;
use std::path::{self, Path, PathBuf};
use std::str::FromStr;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;


fn get_partition_index<T>(vec : &Vec<T>, num_of_divisions : i32) -> Vec<usize>{
    let mut index_vec = Vec::new();
    let num_per_slice = vec.len()/num_of_divisions as usize;
    index_vec.push(0);
    for i in 0..num_of_divisions -1 {
        index_vec.push(num_per_slice*i as usize);
    }
    return index_vec;
}

fn print_lines(vec : &Vec<String>) {
    for line in vec{
        println!("{}", line);
    }
}

fn get_dir_items<T: AsRef<Path>>(dir :T) -> Vec<PathBuf> {
    let dir = dir.as_ref();
    let mut valid : Vec<PathBuf> = Vec::new();
    if dir.is_dir() {
        let result = fs::read_dir(dir);
        match result{
            Ok(read_dir) =>{
                for entry in read_dir{
                    //ignores unable to be read entries
                    match entry{
                        Ok(e) => valid.push(e.path()),
                        Err(_) => ()
                    }
                }
            },
            Err(_) => return valid
        }
    }
    return  valid;
}
//
fn return_matches<T: AsRef<Path>, S : AsRef<str>>(path_list : &Vec<T>, string : &S ) -> Vec<String> {
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

fn search_iteration(string : impl AsRef<str>, remaining : &mut VecDeque<String>, matches_list : &mut Vec<String>) -> Option<Vec<String>>{
    let curr_dir = remaining.pop_front().unwrap();
    let curr_items = get_dir_items(&curr_dir);
    let matches = return_matches(&curr_items, &string.as_ref());
    matches_list.extend(matches.iter().cloned());
    for item in curr_items {
        if item.is_dir() {
            let dir = item.to_str().unwrap_or_default();
            if dir == String::default() {
                continue;
            }
            remaining.push_back(String::from(dir));
        }
    }
    if matches.is_empty() {
        return None;
    }
    return Some(matches);
}

fn st_search<T: AsRef<Path>, S : AsRef<str>>(start : T, string : S, verbose : bool) -> Result<Vec<String>, String>{
    let mut match_list : Vec<String> =Vec::new();
    let mut remaining : VecDeque<String> = VecDeque::new();
    match start.as_ref().to_str() {
        Some(s) => remaining.push_back(String::from(s)),
        None => return Err(String::from("Couldn't start search"))
    }
    while !remaining.is_empty(){
        let matches = search_iteration(&string, &mut remaining, &mut match_list);
        match matches {
            Some(s) => if verbose {print_lines(&s)},
            None => ()
        }
    }
    return Ok(match_list);
}
//multi threaded search, start is initial path, string is the one which you are finding, verbose prints each match as it`s found, times is the number
//of iterations before aggregation with main thread, threads is the thread num
fn mt_search(start : impl AsRef<Path>, string : &'static  String, verbose: bool, times : i32, threads_number : Option<usize>) -> Result<Vec<String>, std::io::Error>{
    let threads : usize;
    match threads_number {
        Some(t) => threads = t,
        None => {
            match thread::available_parallelism() {
                Ok(val) => threads = val.into(),
                Err(e) => return Err(e),
            }
        } 
    }
    let mut match_list : Vec<String> = Vec::new();
    let mut remaining : VecDeque<String> = VecDeque::new();
    match start.as_ref().to_str() {
        Some(s) => remaining.push_back(String::from(s)),
        None => return Ok(Vec::new()),
    }
    let mut handle_vec = Vec::new();
    let mut remaining_ref = &remaining;
    while !remaining.is_empty() {
        //clears so they can be refilled again with the handles and senders
        handle_vec.clear();
        let paths_per_thread = remaining.len() / threads as usize;
        for t in 0..threads {
            let slice_start : usize = paths_per_thread* t as usize;
            let slice_end : usize;
            if t == threads - 1{
                slice_end = remaining.len();
            }
            else {
                slice_end = paths_per_thread*(t as usize + 1 );
            }
            //stores each receiver so it's possible to retrieve the remaining dirs to be explored
            let mut local_remaining = VecDeque::new();
            for i in slice_start..slice_end {
                local_remaining.push_back(remaining[i].clone());
            }
            let string_ref = string;
            
            let handle = thread::spawn(move || {
                let mut i = 0;
                let mut local_matches_list : Vec<String> = Vec::new();
                while !local_remaining.is_empty() && i < times {
                    let matches = search_iteration(string_ref, &mut local_remaining, &mut local_matches_list);
                    
                    i+=1;
                }
                if local_remaining.is_empty() {
                }
                else {
                }
            });
            handle_vec.push(handle);
        }
        remaining.clear();
        //colects output from threads
        for i in 0..threads{
            let received = rx_vec[i as usize].recv();
            //attempts to get value
            || -> Option <()>{
                match received {
                    Ok(value) => remaining.extend(value?),
                    Err(_) => ()
                }
                return Some(());
            }();
            
        }
        handle_vec.drain(0..(threads as usize)).for_each(|x| {let _ = x.join();});
    }
    return Ok(match_list);
}

fn main() {
    match st_search("C:\\Program Files (x86)", "ab", true) {
        Ok(_) => (),
        Err(error) => println!("{}", error)
     }
}
