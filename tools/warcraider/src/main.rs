use std::io;
use std::fs;
use std::path;
use std::collections::HashMap;
use std::io::Read;
use std::env;

use rust_warc::WarcReader;
use avro_rs::{Schema, Writer, types::Record, to_value};
use failure::Error;
use rake::*;
use libflate::gzip::Decoder;
use soup::*;
use regex::*;
use rayon::prelude::*;
use subprocess::{Exec, Redirection,ExitStatus};
use stackdriver_logger;
use log::{error, info, trace, debug, warn};
// run in prod:
// RUSTFLAGS="-C target-cpu=native" cargo run --release

fn main() -> Result<(), Error> {
stackdriver_logger::init_with_cargo!();

let mut warc_number: usize;
match env::var("WARC_NUMBER") {
    Ok(val) => warc_number= val.parse::<usize>().unwrap(),
    Err(_e) => warc_number=1,
}
match env::var("OFFSET") {
    Ok(val) => warc_number=warc_number+ val.parse::<usize>().unwrap(),
    Err(_e) => warc_number=warc_number+0,
}
let offset: usize;
match env::var("REPLICAS") {
    Ok(val) => offset= val.parse::<usize>().unwrap(),
    Err(_e) => offset=1,
}

    let ga_regex = Regex::new(r"\bUA-\d{4,10}-\d{1,4}\b|\bGTM-[A-Z0-9]{1,7}\b").unwrap();
    let whitespace_regex = Regex::new(r"\s+").unwrap();
    let raw_schema = r#"
        {                                                                                                        
    "name": "url_resource",                                                                                                                        
    "type": "record",                                                                                                                              
    "fields": [                                                                                                                                    
        {"name": "url", "type": "string"},                                                                                                         
        {"name": "size_bytes", "type": "int"},                                                                                                     
        {"name": "load_time", "type": "float"},                                                                                                    
        {"name": "title", "type": "string"},                                                                                                       
        {"name": "google_analytics", "type": "string"},                                                                                            
        {"name": "text_content", "type": "string"},                                                                                                
        {"name": "headings_text", "type": "string"},                                                                                               
        {"name": "word_count", "type": "int"},                                                                                                     
        {"name": "links", "type": {"type": "array", "items": "string"}},                                                                           
        {"name": "resource_urls", "type": {"type": "array", "items": "string"}},                                                                   
        {"name": "keywords", "type": {"type": "map", "values": "float"}},                                                                          
        {"name": "meta_tags", "type": {"type": "map", "values": "string"}},                                                                        
        {"name": "headers", "type": {"type": "map", "values": "string"}}                                                                           
    ]   
        }
    "#;

    let schema = Schema::parse_str(raw_schema)?;

while warc_number < 86 {
if warc_number == 59 {
warn!("404 not found");
warc_number=warc_number + offset;
}
let urls = vec!["https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/75697463-298e-4e98-8e41-b6d364e38e1d/download/dta-report02-1.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/af8159f8-b7e0-4c9b-8086-2b0e5b21cb2c/download/dta-report02-2.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/5d10be96-2974-494f-af9a-7e66a093c2ed/download/dta-report02-3.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/720f3cb2-f4e3-4f52-9557-7d919aef5b8d/download/dta-report02-4.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/5c0b0e00-607d-48f4-9138-75615b25d524/download/dta-report02-5.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/2ee1363c-48b1-46bc-88e2-5aefe7f32238/download/dta-report02-6.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/be6c55b5-e6b8-4088-9d70-b2315f1d6eeb/download/dta-report02-7.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/3056bd28-2a52-414f-bb0c-a9fa79ba8c32/download/dta-report02-8.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/9d2e3997-8beb-4dd4-bdde-b58e17e454bb/download/dta-report02-9.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/1b671d71-c5dd-4862-83c1-61974fb58408/download/dta-report02-10.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/b050628b-d215-4fe1-b316-3204fda4f107/download/dta-report02-11.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/fb4a5305-fbc3-4f35-93da-efed376af96b/download/dta-report02-12.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/dc883445-b8ac-48c2-8e48-7b993f7626d8/download/dta-report02-13.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/0ad24884-ff23-4963-b110-e2ddc708c85c/download/dta-report02-14.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/74feb0f7-3d51-485e-b4d0-3d0f3e1917f4/download/dta-report02-15.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/fb2ef784-da34-4e55-9d3e-96e38f985441/download/dta-report02-16.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/70c4443e-6834-4739-bd31-65cbd321fa20/download/dta-report02-17.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/72b917ff-aa77-4af2-9305-804977d15f07/download/dta-report02-18.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/8e593405-1f10-4217-85be-2201ca29e827/download/dta-report02-19.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/d1e1e400-c3ec-4b3c-be44-b3a36bf6c2e8/download/dta-report02-20.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/e9702e88-aeea-43cb-aa49-98e29fed9919/download/dta-report02-21.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/d77e61c1-6795-4ba6-9704-e5166bad719b/download/dta-report02-23.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/2435f1af-c811-4d83-b765-e37a091a4e05/download/dta-report02-24.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/e09aa564-5470-4aeb-bf2b-85bc4392225b/download/dta-report02-25.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/6784f99f-1d22-4874-be7a-0a077b24cce0/download/dta-report02-25.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/90dfc66f-82f0-4aed-ab61-8e9a82914449/download/dta-report02-26.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/87d22e9d-b406-4211-91ce-a381b302307c/download/dta-report02-26.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/c57f6f29-b700-4127-83c7-308dfb956018/download/dta-report02-27.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/1de7225f-c849-4dcd-8304-bcfe476c2e5c/download/dta-report02-28.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/af10074a-a453-48ac-9d1d-4dd92007b6da/download/dta-report02-29.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/bd32d4a0-17c4-4d61-8684-fb12c99da9c6/download/dta-report02-30.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/3ce01826-b67d-40d4-b458-e5c059f44fcc/download/dta-report02-31.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/df480352-a377-4543-817c-dc42259ceeca/download/dta-report02-32.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/4bc93329-4c56-4b8a-9c10-e8df62554f89/download/dta-report02-34.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/214f7b4a-5614-4cff-ad64-7707cab2896c/download/dta-report02-35.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/f0080855-9be1-4a86-844c-23d7a10f8d7b/download/dta-report02-36.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/49c19cea-e1da-49d8-90d1-80307901a90b/download/dta-report02-37.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/bc9f675d-dfac-462a-9565-e311727e33aa/download/dta-report02-38.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/902f9620-fc31-4e9b-bf8f-71665f66d9d4/download/dta-report02-39.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/6f624960-c9cd-467c-9caf-e786602a2657/download/dta-report02-40.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/9b76accf-506c-4ceb-8efd-d9ba18ad6c8a/download/dta-report02-41.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/7bbdaffe-391b-4d82-b458-85b097cd8578/download/dta-report02-42.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/3935c6eb-1afa-4539-97cd-52e6399cc1a3/download/dta-report02-42.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/3e587a7a-626b-400f-ac17-5a96962255cb/download/dta-report02-44.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/ecf5479a-8d82-42f6-98f1-a44bfb1c9b8a/download/dta-report02-45.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/a36765e5-9888-4c02-9689-70001f257ee2/download/dta-report02-46.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/30bdd72b-e2e1-4278-b15e-08dfbad2291d/download/dta-report02-47.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/294705a3-74ea-4322-86c8-8466b8e8ceb4/download/dta-report02-48.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/bb0c6d79-b239-4d37-8101-932e10c17964/download/dta-report02-48.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/dfd8378f-e7fe-411c-af1c-dafd113f389a/download/dta-report02-49.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/04269174-a6b3-4443-b813-08a42cbfce4d/download/dta-report02-50.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/6e09de95-5157-48fc-9da7-f7e840f07b9d/download/dta-report02-51.warc",
"https://data.gov.au/data/dataset/99f43557-1d3d-40e7-bc0c-665a4275d625/resource/a07de461-ab6e-486a-85b3-d0d7b195d521/download/dta-report02-52.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-53.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-54.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-55.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-56.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-57.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-58.warc",
"",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-60.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-61.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-62.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-63.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-64.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-65.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-66.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-67.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-68.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-69.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-70.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-71.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-72.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-73.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-74.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-75.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-76.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-77.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-78.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-79.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-80.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-81.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-82.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-83.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-84.warc",
"https://datagovau.s3.ap-southeast-2.amazonaws.com/cd574697-6734-4443-b350-9cf9eae427a2/99f43557-1d3d-40e7-bc0c-665a4275d625/dta-report02-85.warc"];
let avro_filename = String::from("") + "dta-report02-" + warc_number.to_string().as_str() + ".avro";
    let file = fs::File::create(&avro_filename)?;
let avro_gcs_status = Exec::cmd("gsutil").arg("stat").arg(String::from("gs://us-east1-dta-airflow-b3415db4-bucket/data/bqload/")+&avro_filename).join()?;
info!("");
if avro_gcs_status == ExitStatus::Exited(0) {
warn!("avro does exist on gcs {}",avro_filename);
} else {
info!("avro does not exist on gcs {}",avro_filename);

    let mut writer = Writer::new(&schema, file);
// download file
let warc_filename = String::from("") + "dta-report02-" + warc_number.to_string().as_str() + ".warc";
if !path::Path::new(&warc_filename).exists() {

    info!("starting download: {}",urls[warc_number]);
    let mut response = reqwest::get(urls[warc_number])?;
    let mut dest = fs::File::create(&warc_filename)?;
    io::copy(&mut response, &mut dest)?;
       debug!("downloaded");
}
    let f = fs::File::open(&warc_filename)
        .expect("Unable to open file");
    let br = io::BufReader::new(f);

    let warc = WarcReader::new(br);
    let mut i = 0;
    for item in warc {
        i += 1;
        if i < 4 {
            let warc_record = item.unwrap(); // could be IO/malformed error

            if warc_record.header.get(&"WARC-Type".into()) == Some(&"response".into()) {
                let url = warc_record
                    .header
                    .get(&"WARC-Target-URI".into())
                    .unwrap()
                    .as_str();
                let size = warc_record
                    .header
                    .get(&"Uncompressed-Content-Length".into())
                    .unwrap_or(&String::from("0"))
                    .parse::<i32>()
                    .unwrap();
                info!("{} {} ({} bytes)", i, url, size);

                if size > 1000000 {
                    warn!("too big {}", url);
                } else {
                    let mut record = Record::new(writer.schema()).unwrap();
                    record.put("url", url);
		    let mut content = String::new();
                    let mut decoder = Decoder::new(&warc_record.content[..]).unwrap();
decoder.read_to_string(&mut content)?;
                    let parts: Vec<&str> = content.split("\n\r\n").collect();
                    let raw_html = String::from(parts[1]);
                    let mut headers = HashMap::<String, String>::new();
                    for line in parts[0].split("\n") {
                            if line == "" || line.starts_with("HTTP/") {
                            } else if line.contains(": ") {
                                let parts: Vec<&str> = line.split(": ").collect();
                                headers.insert(String::from(parts[0]), String::from(parts[1]));
                            } 
                    }

                    record.put("size_bytes", size);

                    record.put(
                        "load_time",
                        headers
                            .get("X-Funnelback-Total-Request-Time-MS")
                            .unwrap_or(&String::from(""))
                            .as_str()
                            .parse::<f32>()
                            .unwrap_or(0.0)
                            / 1000.0,
                    );
                    record.put("headers", to_value(headers).unwrap());
                    let soup = Soup::new(&raw_html);
                    let text = whitespace_regex
                        .replace_all(
                            soup.tag("body")
                                .find()
                                .unwrap()
                                .children()
                                .map(|tag| {
                                    if tag.name().to_string() == "script"
                                        || tag.name().to_string() == "noscript"
                                        || tag.name().to_string() == "style"
                                    {
                                        String::from("")
                                    } else {
                                        tag.text().trim().to_string()
                                    }
                                })
                                .collect::<Vec<String>>()
                                .join("")
                                .as_str(),
                            " ",
                        )
                        .to_string();
                    let text_words = String::from("") + text.as_str();
                    match soup.tag("title").find() {
                        Some(title) => record.put("title", title.text().trim()),
                        None => record.put("title", ""),
                    }

                    record.put("text_content", text);
                    record.put("word_count", *&text_words.par_split_whitespace().count() as i32);

                    match ga_regex.captures(&raw_html) {
                        Some(caps) => record.put("google_analytics", caps.get(0).unwrap().as_str()),
                        None => record.put("google_analytics", ""),
                    }


                    let mut headings_text = String::new();
                    for heading in vec!["h1", "h2", "h3", "h4", "h5", "h6"].iter() {
                        for header in soup.tag(*heading).find_all() {
                            let head_text = header.text();
                            if head_text.len() > 0 {
                                headings_text.push('\n');
                                headings_text.push_str(&head_text);
                            }
                        }
                    }
                    record.put("headings_text", headings_text);

                    record.put(
                        "links",
                        to_value(
                            soup.tag("a")
                                .find_all()
                                .filter_map(|link| link.get("href"))
                                .collect::<Vec<_>>(),
                        )
                        .unwrap(),
                    );

                    let resource_urls: Vec<String> = [
                        soup.tag("script")
                            .find_all()
                            .filter_map(|link| link.get("src"))
                            .collect::<Vec<String>>(),
                        soup.tag("link")
                            .find_all()
                            .filter_map(|link| link.get("href"))
                            .collect::<Vec<String>>(),
                        soup.tag("img")
                            .find_all()
                            .filter_map(|link| link.get("src"))
                            .collect::<Vec<String>>(),
                    ]
                    .concat();
                    record.put("resource_urls", to_value(resource_urls).unwrap());

                    let mut meta_tags = HashMap::<String, String>::new();
                    soup.tag("meta").find_all().for_each(|meta| {
                        let attrs = meta.attrs();
                        if attrs.contains_key("name") {
                            match attrs.get("content") {
                                Some(i) => meta_tags
                                    .insert(attrs.get("name").unwrap().to_string(), i.to_string()),
                                None => Some(String::from("?")),
                            };
                        } else if attrs.contains_key("http-equiv") {
                            //If http-equiv is set, it is a pragma directive — information normally given by the web server about how the web page is served.
                            match attrs.get("content") {
                                Some(i) => meta_tags.insert(
                                    attrs.get("http-equiv").unwrap().to_string(),
                                    i.to_string(),
                                ),
                                None => Some(String::from("?")),
                            };
                        } else if attrs.contains_key("charset") {
                            //If charset is set, it is a charset declaration — the character encoding used by the webpage.
                            meta_tags.insert(
                                String::from("charset"),
                                attrs.get("charset").unwrap().to_string(),
                            );
                        } else if attrs.contains_key("itemprop") {
                            //If itemprop is set, it is user-defined metadata — transparent for the user-agent as the semantics of the metadata is user-specific.
                            match attrs.get("content") {
                                Some(i) => meta_tags.insert(
                                    attrs.get("itemprop").unwrap().to_string(),
                                    i.to_string(),
                                ),
                                None => Some(String::from("?")),
                            };
                        } else if attrs.contains_key("property") {
                            //facebook open graph

                            match attrs.get("content") {
                                Some(i) => meta_tags.insert(
                                    attrs.get("property").unwrap().to_string(),
                                    i.to_string(),
                                ),
                                None => Some(String::from("?")),
                            };
                        }
                    });
                    record.put("meta_tags", to_value(meta_tags).unwrap());

                    let sw = StopWords::from_file("SmartStoplist.txt").unwrap();
                    let r = Rake::new(sw);
                    let mut keywords = HashMap::<String, f32>::new();
                    r.run(text_words.as_str()).iter().for_each(
                        |&KeywordScore {
                             ref keyword,
                             ref score,
                         }| {
                            keywords.insert(String::from("")+keyword.as_str(), *score as f32);
                        },
                    );
                    record.put("keywords", keywords);
                    //dbg!(record);
                    writer.append(record)?;
                    writer.flush()?;
                }
            }
        }
    }
let upload = Exec::cmd("gsutil").arg("cp").arg(&avro_filename).arg(String::from("gs://us-east1-dta-airflow-b3415db4-bucket/data/bqload/")+&avro_filename)
            .stdout(Redirection::Pipe)
            .capture().unwrap()
            .stdout_str();
        info!("{:?}", upload);
}
warc_number = warc_number + offset;
}
    Ok(())
}
