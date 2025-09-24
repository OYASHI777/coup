use std::collections::HashSet;

// To compartmentalise and put string sorts into sort_str

pub fn unique_sorted_string(tokens: &str) -> Vec<char> {
    // Takes in tokens and returns a String of unique values that have been sorted

    // Removes duplicates
    let mut chars: Vec<char> = tokens.chars().collect::<HashSet<_>>().into_iter().collect();
    chars.sort_unstable();
    chars
}

pub fn filter_string_less_than(unique_sorted_string: &[char], char_max: &char) -> Vec<char> {
    // Takes a unique sorted string and removes all items above a given character char_max
    // unique sorted string should have no duplicates and be ascending
    let mut l: usize = 0;
    let mut r: usize = unique_sorted_string.len() - 1;
    let mut mid: usize;
    // let unique_sorted_string: Vec<char> = unique_sorted_string.collect();

    // Find the mid such that we can slice the string (excluding the mid value)
    // SO if char_max is in the string, return it else return the length of the string
    if unique_sorted_string[r] <= *char_max {
        return unique_sorted_string.to_vec();
    }
    if unique_sorted_string[l] > *char_max {
        return Vec::new();
    }
    if unique_sorted_string[l] == *char_max {
        return unique_sorted_string[..(l + 1)].to_vec();
    }
    while l < r {
        mid = (l + r) / 2;
        if unique_sorted_string[mid] < *char_max {
            l = mid + 1;
        } else if unique_sorted_string[mid] > *char_max {
            r = mid - 1;
        } else {
            l = mid + 1;
            r = l;
        }
    }
    unique_sorted_string[..l].to_vec()
}

pub fn filter_string_more_than(unique_sorted_string: &[char], char_min: &char) -> Vec<char> {
    // Takes a unique sorted string and removes all items below a given character char_max
    // unique sorted string should have no duplicates and be ascending
    let mut l: usize = 0;
    let mut r: usize = unique_sorted_string.len() - 1;
    let mut mid: usize;
    // let chars: Vec<char> = unique_sorted_string.collect();

    // Find the mid such that we can slice the string (excluding the mid value)
    // SO if char_max is in the string, return it else return the length of the string
    if unique_sorted_string[l] >= *char_min {
        return unique_sorted_string.to_vec();
    }
    if unique_sorted_string[r] < *char_min {
        return Vec::new();
    }
    if unique_sorted_string[r] == *char_min {
        return unique_sorted_string[r..].to_vec();
    }
    while l < r {
        mid = (l + r) / 2;
        if unique_sorted_string[mid] < *char_min {
            l = mid + 1;
        } else if unique_sorted_string[mid] > *char_min {
            r = mid - 1;
        } else {
            l = mid;
            r = l;
        }
    }
    unique_sorted_string[l..].to_vec()
}

pub fn filter_string_within(
    unique_sorted_string: &[char],
    char_min: &char,
    char_max: &char,
) -> Vec<char> {
    let output: Vec<char> = filter_string_less_than(unique_sorted_string, char_max);
    filter_string_more_than(&output, char_min)
}

pub fn sort_str(string: &str) -> String {
    let mut sorted_string = string.to_string();
    let mut chars: Vec<char> = sorted_string.chars().collect();
    chars.sort();
    sorted_string = chars.into_iter().collect();
    sorted_string
}

pub fn remove_chars(string: &str, chars_to_remove: &str) -> String {
    let mut result_string = string.to_string();
    for char_to_remove in chars_to_remove.chars() {
        if let Some(index) = result_string.find(char_to_remove) {
            result_string.remove(index);
        }
    }
    result_string
}

pub fn replace_chars(string: &str, chars_to_remove: &str, char_to_replace: &str) -> String {
    //replaces and sorts string
    let mut result_string = string.to_string();
    for char_to_remove in chars_to_remove.chars() {
        if let Some(index) = result_string.find(char_to_remove) {
            result_string.remove(index);
        }
    }
    result_string += char_to_replace;
    let mut temp: Vec<char> = result_string.chars().collect();
    temp.sort_unstable();
    result_string = temp.into_iter().collect();
    result_string
}

pub fn contains_all_chars(string1: &str, string2: &str) -> bool {
    // This is faster
    let mut s1: String = string1.to_string();

    for c in string2.chars() {
        if let Some(index) = s1.find(c) {
            s1.remove(index);
        } else {
            return false;
        }
    }
    true
}
