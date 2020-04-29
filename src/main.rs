
use rust_stemmers::{Algorithm, Stemmer}; // for stemming single words
use seahash::hash as shash; // a fast, portable hash library
use std::collections::HashMap; // for dictionaries



fn text_bin_counts(text: String) -> HashMap<u64, f32> {
    // text goes in, sparse vector comes out
        
    // declare some variables and bring them into context
    let en_stemmer = Stemmer::create(Algorithm::English); // english langage stemmer- one word at a time please!
    let mut h1: u64;        // the hash of a 1-gram
    let mut bin1: u64;      // the "bin" (remainder) of h1
    let mut h2: u64;        // the hash of a 2-gram
    let mut bin2: u64;      // the "bin" (remainder) of h2
    let mut stem: std::borrow::Cow<str>; // copy-on-write pointer to a word stem
    let mut bigram = String::from("!NewDoc"); // bigram of last two grams
    let mut bin_counts: HashMap<u64, f32> = HashMap::new(); // a "sparse vector" for counts fo binned hashes

    // convert text to lower case and iterate over words
    let low_string = text.to_lowercase();
    let words = low_string.split_whitespace();
    for gram in words  {

        // find the word stem and bigram with the previous stem
        stem = en_stemmer.stem(&gram);
        bigram.push_str(" ");
        bigram.push_str(&stem);

        // hash the stem, find its bin, and increment bin_counts
        h1 = shash( &stem.as_bytes() );
        bin1 = h1 % 3000000;
        // get a pointer to the value, inserting 0 if it doesn't exist, and increment by 1
        *bin_counts.entry(bin1).or_insert(0f32)+=1f32; 

        h2 = shash( &bigram.as_bytes() );
        bin2 = h2 % 3000000;
        // get a pointer to the value, inserting 0 if it doesn't exist, and increment by 1
        *bin_counts.entry(bin2).or_insert(0f32)+=1f32; 
        
        //println!("word='{}' stem='{}' bigram='{}' bin_word={} bin_bigram={}", &gram, &stem, &bigram, bin1, bin2);
        // replace the "previous gram" so bigram is ready for the next loop
        bigram = stem.to_string();        
    }
    // return the sparse vector
    bin_counts
}


fn cosine_similarity(u: HashMap<u64, f32>, v: HashMap<u64, f32>) -> f32 {
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
    let similarity:f32 = (100.0f32*dot_prod/(u_norm * v_norm)).into(); // as percentage
    //println!("u_norm={}, v_norm={}, dot_prod={}", u_norm, v_norm, dot_prod);
    similarity

}
fn main() {
    
    let vec1 = text_bin_counts("I wanna play all day".to_string());
    let vec2 = text_bin_counts("All work and no play makes for a sad day".to_string());

    let sim = cosine_similarity(vec1, vec2);
    println!("similarity={}", sim);

}