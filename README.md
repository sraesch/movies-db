# Movies DB
Movies DB is a simple solution for storing, managing, searching, and visualizing movies. With an intuitive web UI, dive right in to effortlessly organize and explore your movie collection. Movies DB is designed to be very simple and does not have a user understanding.

## Run

### Requirements
You

### How to run it
Checkout the helm charts
```bash
git clone https://github.com/sraesch/movies-db.git
cd movies-db/helm
helm install movies-db .
```
Note, you might have to change/update the values.yaml before you install it.

## Development
In order to setup your development environment, do the following steps.

### Compile and run the backend service
#### Requirements
* Rust: You need to install rust and make it available. See https://www.rust-lang.org/tools/install
* ffmpeg: In order to run the service, you'll need to have ffmpeg installed.

#### Compile dev build
In order to build the service with debug symbols, run
```bash
cd movies-db-service
cargo build
```
In order to build the release build, add the `--release` option:
```bash
cd movies-db-service
cargo build --release
```
Optionally, you can run all the tests by running the following command:
```bash
cd movies-db-service
cargo test
```

#### Run the backend service
After the backend service has been built and if ffmpeg is installed, you can run the service with the following command:
```bash
cd movies-db-service
cargo test
```bash
./movies-db-service/target/debug/movies-db-cli --root-dir ./temp --ffmpeg /usr/bin
```
We assume the binary of ffmpeg is located in `/usr/bin`. If not, please change the path accordingly.
You can check for further options with `--help`.

### Compile and run UI
#### Requirements
* In order to compile and run the UI, you'll need a recent version of `nodejs` and `npm`.

#### Compile dev build
Before you can run the UI, make sure the backend service is running and update `movies-db-ui/.env` file to specify the endpoint
of your backend service. Usually, this will be:
```bash
REACT_APP_SERVER_ADDRESS=http://localhost:3030/api/v1
```

In order to start the UI in development mode run the following commands:
```bash
cd movies-db-ui
npm install
npm run start
```
Afterwards, you're system should be fully available for development.
In order to build the release version, run the following command:
```bash
cd movies-db-ui
npm install
npm run build
```