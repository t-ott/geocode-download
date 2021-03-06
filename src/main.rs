extern crate dotenv;

use dotenv::dotenv;
use std::env;
use structopt::StructOpt;
use reqwest::Url;

const GEOCODE_BASE_URL: &str = "https://maps.googleapis.com/maps/api/geocode/json?";
const PARCELS_BASE_URL: &str = "https://services1.arcgis.com/BkFxaEFNwHqX3tAw/arcgis/rest/services/FS_VCGI_OPENDATA_Cadastral_VTPARCELS_poly_standardized_parcels_SP_v1/FeatureServer/0/query?";

// Command line arguments
#[derive(StructOpt)]
struct Cli {
    address: String
}

fn main() {
    dotenv().ok();
    let key = "GOOGLE_GEOCODING_API_KEY";
    let geocode_api_key = match env::var(key) {
        Ok(v) => v,
        Err(_e) => panic!("${} is not set", key)
    };

    let args = Cli::from_args();
    let geocoding_url = Url::parse_with_params(
        &GEOCODE_BASE_URL,
        [
            ("address", &args.address),
            ("key", &geocode_api_key.to_string())
        ]
    );

    match geocoding_url {
        Ok(url) => {
            let json_text = get_geocoding(url);
            let json: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&json_text);
            match json {
                Ok(json) => {
                    let bbox: [std::string::String; 4] = parse_geocoding(json);
                    get_parcels(bbox)
                }
                Err(_) => println!("JSON parsing error!")
            }
        }
        Err(_) => println!("URL parsing error!")
    }
}

fn get_geocoding(url: Url) -> std::string::String {
    println!("Sending request to Google Geocoding API...");
    // println!("Sending get request to: {}", url.as_str());
    match reqwest::blocking::get(url) {
        Ok(response) => {
            println!("Got response!");
            if response.status() == reqwest::StatusCode::OK {
                match response.text() {
                    Ok(text) => text,
                    Err(_) => "Could not get response text!".to_string()
                }
            }
            else {
                "Oops. Response status not OK.".to_string()
            }
        }
        Err(_) => "Oops. Did not get response.".to_string()
    }
}

fn parse_geocoding(json: serde_json::Value) -> [std::string::String; 4]{
    // println!("{}", json.to_string())
    let xmin = &json["results"][0]["geometry"]["viewport"]["southwest"]["lng"];
    let ymin = &json["results"][0]["geometry"]["viewport"]["southwest"]["lat"];
    let xmax = &json["results"][0]["geometry"]["viewport"]["northeast"]["lng"];
    let ymax = &json["results"][0]["geometry"]["viewport"]["northeast"]["lat"];
    let bbox: [std::string::String; 4] = [xmin.to_string(), ymin.to_string(), xmax.to_string(), ymax.to_string()];
    bbox
}

fn get_parcels(bbox: [std::string::String; 4]) {
    // println!("{}", bbox.join(","));

    let bbox = bbox.join(",");
    let parcel_url = Url::parse_with_params(&PARCELS_BASE_URL, &[
        ("where", "1=1"),
        ("outFields", "*"),
        ("geometry", &bbox), // xmin, ymin, xmax, ymax
        ("geometryType", "esriGeometryEnvelope"),
        ("inSR", "4326"),
        ("spatialRel", "esriSpatialRelIntersects"),
        ("outSR", "4326"),
        ("f", "json")
    ]);

    match parcel_url {
        Ok(url) => {
            println!("Sending request to VCGI API...");
            // println!("URL: {}", url.as_str());
            match reqwest::blocking::get(url) {
                Ok(response) => {
                    println!("Got response!");
                    if response.status() == reqwest::StatusCode::OK {
                        match response.text() {
                            // Ok(text) => println!("{}", text),
                            Ok(text) => {
                                std::fs::write("parcels.geojson", text).ok();
                                println!("Data written to parcels.geojson")
                            }
                            Err(_) => println!("Oops! Could not get response text.")
                        }
                    }
                    else { println!("Oops! Response status not OK.") }
                }
                Err(_) => println!("Oops! Did not get response.")
            }
        }
        Err(_) => println!("URL parsing Error!")
    }
}
