use structopt::StructOpt;
use compare_csv::app::App;
use compare_csv::core::TableComparator;
use owo_colors::OwoColorize;

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    env_logger::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let app = App::from_args();
    let comp = TableComparator::from_app(&app)?;

    for p in comp.different_points().iter() {
        println!("{}", format!("# {}", p).bold());
        let (src, dst) = comp.get_records(p);
        for x in src.iter() {
            println!("{}", format!("< {}", x).red());
        }
        for x in dst.iter() {
            println!("{}", format!("> {}", x).green());
        }
    }

    Ok(())
}
