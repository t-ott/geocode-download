extern crate dotenv;

use dotenv::dotenv;
use reqwest::Url;
use std::env;
use std::error::Error;
use std::string::String;
use structopt::StructOpt;

const GEOCODE_URL: &str = "https://maps.googleapis.com/maps/api/geocode/json?";
const PARCELS_URL: &str = "https://services1.arcgis.com/BkFxaEFNwHqX3tAw/arcgis/rest/\
    services/FS_VCGI_OPENDATA_Cadastral_VTPARCELS_poly_standardized_parcels_SP_v1/\
    FeatureServer/0/query?";

// Command line arguments
#[derive(StructOpt)]
struct Cli {
    address: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let key = "GOOGLE_GEOCODING_API_KEY";
    let geocode_api_key = match env::var(key) {
        Ok(v) => v,
        Err(_e) => panic!("${} is not set", key),
    };
    let args = Cli::from_args();

    let geocoding_url = Url::parse_with_params(
        &GEOCODE_URL,
        [
            ("address", &args.address),
            ("key", &geocode_api_key.to_string()),
        ],
    )
    .expect("Failed to parse geocoding URL params");

    let json_text = get_geocoding(geocoding_url)?;
    let json: serde_json::Value = serde_json::from_str(&json_text)?;
    let bbox = parse_geocoding(json);
    get_parcels(bbox)?;
    Ok(())
}

fn get_geocoding(url: Url) -> Result<String, reqwest::Error> {
    println!("Sending request to Google Geocoding API...");
    let response = reqwest::blocking::get(url)?;
    println!("Got response.");
    let json_text = response.text()?;
    Ok(json_text)
}

fn parse_geocoding(json: serde_json::Value) -> [String; 4] {
    if json["results"].as_array().unwrap().len() == 0 {
        panic!("Google Geocoding did return any results")
    }
    // Extract a local bounding box from response
    let xmin = &json["results"][0]["geometry"]["viewport"]["southwest"]["lng"];
    let ymin = &json["results"][0]["geometry"]["viewport"]["southwest"]["lat"];
    let xmax = &json["results"][0]["geometry"]["viewport"]["northeast"]["lng"];
    let ymax = &json["results"][0]["geometry"]["viewport"]["northeast"]["lat"];
    let bbox: [String; 4] = [
        xmin.to_string(),
        ymin.to_string(),
        xmax.to_string(),
        ymax.to_string(),
    ];
    bbox
}

fn get_parcels(bbox: [String; 4]) -> Result<(), reqwest::Error> {
    // Send request to VCGI API for parcel data
    let bbox = bbox.join(",");
    let parcel_url = Url::parse_with_params(
        &PARCELS_URL,
        &[
            ("where", "1=1"),
            ("outFields", "*"),
            ("geometry", &bbox), // xmin, ymin, xmax, ymax
            ("geometryType", "esriGeometryEnvelope"),
            ("inSR", "4326"),
            ("spatialRel", "esriSpatialRelIntersects"),
            ("outSR", "4326"),
            ("f", "json"),
        ],
    )
    .expect("Failed to parse parcel URL params");

    println!("Sending request to VCGI API...");
    let response = reqwest::blocking::get(parcel_url)?;
    println!("Got response.");
    let response_text = response.text()?;
    std::fs::write("parcels.geojson", response_text).ok();
    println!("Parcels written to parcels.geojson!");
    Ok(())
}
