use structopt::StructOpt;
use compare_csv::app::App;
use compare_csv::core::TableComparator;
use ansi_term::Style;
use ansi_term::Colour::{Red, Green};

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    env_logger::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let app = App::from_args();
    let comp = TableComparator::from_app(&app)?;

    let (bold, red, green) = if atty::is(atty::Stream::Stdout) {
        (Style::new().bold(), Red.normal(), Green.normal())
    } else {
        (Style::new(), Style::new(), Style::new())
    };

    for p in comp.different_points().iter() {
        println!("{}", bold.paint(format!("# {}", p)));
        let (src, dst) = comp.get_records(p);
        for x in src.iter() {
            println!("{}", red.paint(format!("< {}", x)));
        }
        for x in dst.iter() {
            println!("{}", green.paint(format!("> {}", x)));
        }
    }

    Ok(())
}
