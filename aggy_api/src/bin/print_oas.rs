use deps::*;

use aggy_api::*;

fn main() {
    println!(
        "{}",
        <ApiDoc as utoipa::OpenApi>::openapi()
            .to_pretty_json()
            .unwrap()
    );
}
