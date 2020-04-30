
use rust_stemmers::{Algorithm, Stemmer}; // for stemming single words
use seahash::hash as shash; // a fast, portable hash library
use std::collections::HashMap; // for dictionaries
use std::fs::File; // for creating and writing files
use std::io::prelude::*; // allows File.write_all

const BIN_DIVISIONS: u64 = 40_000_000;
const DOC_FREQ_FILE: &str = "WikiDocFreq_40m";

fn text_bin_counts(text: String) -> HashMap<u32, f32> {
    // text goes in, sparse vector comes out
        
    // declare some variables and bring them into context
    let en_stemmer = Stemmer::create(Algorithm::English); // english langage stemmer- one word at a time please!
    let mut h1: u64;        // the hash of a 1-gram
    let mut bin1: u32;      // the "bin" (remainder) of h1
    let mut h2: u64;        // the hash of a 2-gram
    let mut bin2: u32;      // the "bin" (remainder) of h2
    let mut stem: std::borrow::Cow<str>; // copy-on-write pointer to a word stem
    let mut bigram = String::from("!NewDoc"); // bigram of last two grams
    let mut bin_counts: HashMap<u32, f32> = HashMap::new(); // a "sparse vector" for counts fo binned hashes

    // convert text to lower case and iterate over words
    let text_clean = text.replace(&['(', ')', ',', '\"', '.', ';', ':', '\''][..], "");
    let text_lower = text_clean.to_lowercase();
    let words = text_lower.split_whitespace();
    for gram in words  {

        // find the word stem and bigram with the previous stem
        stem = en_stemmer.stem(&gram);
        bigram.push_str(" ");
        bigram.push_str(&stem);

        // hash the stem, find its bin, and increment bin_counts
        h1 = shash( &stem.as_bytes() );
        bin1 = (h1 % BIN_DIVISIONS) as u32;
        // get a pointer to the value, inserting 0 if it doesn't exist, and increment by 1
        *bin_counts.entry(bin1).or_insert(0f32)+=1f32; 

        h2 = shash( &bigram.as_bytes() );
        bin2 = (h2 % BIN_DIVISIONS) as u32;
        // get a pointer to the value, inserting 0 if it doesn't exist, and increment by 1
        *bin_counts.entry(bin2).or_insert(0f32)+=1f32; 
        
        //println!("word='{}' stem='{}' bigram='{}' bin_word={} bin_bigram={}", &gram, &stem, &bigram, bin1, bin2);
        // replace the "previous gram" so bigram is ready for the next loop
        bigram = stem.to_string();        
    }
    // return the sparse vector
    bin_counts
}


fn cosine_similarity(u: HashMap<u32, f32>, v: HashMap<u32, f32>) -> f32 {
    // return the similarity of two sparse vectors as defined by (u*v)/(||u||*||v||)

    let mut dot_prod: f32 = 0f32;      // dot product
    let mut u_norm: f32 = 0f32;    // norm of vector u
    let mut v_norm: f32 = 0f32;     // norm of vector v

    for (key, u_element) in &u {
        let v_element = match &v.get(&key){
            Some(element) => element,
            None => &0f32,
        };
        dot_prod = dot_prod + (u_element * v_element);
        u_norm = u_norm + u_element;
        println!("{}.{}.{}", key, u_element, v_element);
    }
    for (_, v_element) in &v{
        v_norm = v_norm + v_element;
    }

    // calculate and return the similarity
    let similarity:f32 = 100.0f32*dot_prod/(u_norm * v_norm); // as percentage
    //println!("u_norm={}, v_norm={}, dot_prod={}", u_norm, v_norm, dot_prod);
    similarity

}

fn dump_hashmap(filepath: &String, map: &HashMap<u32, f32>) {
    
    println!("Dumping hashmap to a binary file...");
    let mut file = File::create(filepath).expect("Unable to open file");
    let mut bytes: [u8;4];

    for key in 0..BIN_DIVISIONS {
        let mapkey = key as u32;
        let val = match map.get(&mapkey) {
            Some(v) => v,
            _ => &0f32,
        };
        //println!("{}-{}", mapkey, val);
        bytes = val.to_be_bytes();
        file.write_all(&bytes).expect("unable to write");
        if key % 500000 == 0 {
            println!("{} keys dumped...", key);
        }
    }
    println!("Successfully saved {}", filepath);
}


fn count_doc_freq() {
    


    let mut doc_freq: HashMap<u32, f32> = HashMap::new();
    let mut doc_ct: u32 = 0;
    
    

    let connection = sqlite::open("Wiki16.db").unwrap();
    connection
    .iterate("SELECT id, text FROM documents", |pairs| {
        for &(column, value) in pairs.iter() {
            //println!("{} = {}", column, value.unwrap());
            if column == "id" {
                //println!("{}", value.unwrap());
                let bin_counts = text_bin_counts(value.unwrap().to_string());
                for (bin, count) in bin_counts {
                    //println!("{}.{}", bin, count)
                    *doc_freq.entry(bin).or_insert(0f32)+= 5f32*count; 
                }
                //println!("id {}", &doc_freq.keys().len());

                
                
            }
            if column == "text" {
                //println!("{}",&value.unwrap());
                let bin_counts = text_bin_counts(value.unwrap().to_string());
                for (bin, count) in bin_counts {
                    //println!("{}.{}", bin, count)
                    *doc_freq.entry(bin).or_insert(0f32)+=count; 
                }
                //println!("text {}", &doc_freq.keys().len());
                doc_ct = doc_ct + 1;
                if doc_ct % 1000 == 0 {
                    let key_ct = &doc_freq.keys().len();
                    println!("docs={}, nonzero={}", doc_ct, key_ct);

                

            }

            }
        }
        true
    })
    .unwrap();

    dump_hashmap(&DOC_FREQ_FILE.to_string(), &doc_freq);

}

fn main() {
    //let vec1 = text_bin_counts("I wanna play all day".to_string());
    //let vec2 = text_bin_counts("All work and no play makes for a sad day".to_string());
    //let sim = cosine_similarity(vec1, vec2);
    //println!("similarity={}", sim);
    count_doc_freq();
}