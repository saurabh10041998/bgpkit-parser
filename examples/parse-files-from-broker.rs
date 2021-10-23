use std::io::BufReader;
use bzip2::read::BzDecoder;
use bgpkit_parser::{BgpElem, BgpkitParser};

/// This example shows how use BGPKIT Broker to retrieve a number of data file pointers that matches
/// the time range criteria, and then parse the data files for each one.
///
/// The dependency needed for this example are:
/// ```
/// bzip2="0.4"
/// reqwest = { version = "0.11", features = ["json", "blocking", "stream"] }
/// bgpkit-broker = "0.3.0"
/// ```
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut params = bgpkit_broker::QueryParams::new();
    params = params.start_ts(1634693400);
    params = params.end_ts(1634693400);
    params = params.data_type("update");
    let mut broker = bgpkit_broker::BgpkitBroker::new("https://api.broker.bgpkit.com/v1");
    broker.set_params(&params);

    for item in broker {
        log::info!("downloading updates file: {}", &item.url);
        // read updates data into bytes
        let data_bytes = reqwest::blocking::get(item.url)
            .unwrap().bytes().unwrap().to_vec();
        // create a buffered reader that wraps around a bzip2 decoder
        let reader = BufReader::new(BzDecoder::new(&*data_bytes));
        // create a parser that takes the buffered reader
        let parser = BgpkitParser::new(reader);

        log::info!("parsing updates file");
        // iterating through the parser. the iterator returns `BgpElem` one at a time.
        let elems = parser.into_elem_iter().map(|elem|{
            if let Some(origins) = &elem.origin_asns {
                if origins.contains(&13335) {
                    Some(elem)
                } else {
                    None
                }
            } else {
                None
            }
        }).filter_map(|x|x).collect::<Vec<BgpElem>>();
        log::info!("{} elems matches", elems.len());
    }
}