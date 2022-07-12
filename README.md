# geocode-download
This is a command-line program to get Vermont parcel data for a user-inputed Vermont address. It uses Google's Geocoding API and VCGI's parcel data endpoint. It will save the parcel data to a file ```parcels.geojson```

Requires a Google API Key to access geocoding services. Set the API key in a file called ```.env``` in the project directory, whose contents would look like:
```
GOOGLE_GEOCODING_API_KEY=abc123blahblah
```

Example usage:

Navigate to the build target, then:
```
./geocode-download "170 Carrigan Drive Burlington, VT"
```
