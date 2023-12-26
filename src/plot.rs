use crate::common::*;
use plotters::{
    coord::ranged1d::{DiscreteRanged, KeyPointHint, NoDefaultFormatting, ValueFormatter},
    prelude::*,
};
use std::path::PathBuf;

fn f(d: &i32) -> String {
    "eutohheotu".to_string()
}

fn plot(path: &PathBuf, results: &TasksResult) -> Result<(), Box<dyn std::error::Error>> {
    let root_area = SVGBackend::new(&path, (320, 200)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root_area)
        .caption("hi", ("inter", 30.0).into_font())
        .margin(10)
        .set_label_area_size(LabelAreaPosition::Left, 8.percent())
        .set_label_area_size(LabelAreaPosition::Bottom, 8.percent())
        .set_label_area_size(LabelAreaPosition::Right, 8.percent())
        .build_cartesian_2d((0..12), 0..100)?;

    chart
        .configure_mesh()
        .x_labels(12)
        .y_labels(10)
        .x_label_formatter(&f)
        .x_desc("Date")
        .y_desc("Beneficiaires")
        // .x_max_light_lines(max_lines)
        //.y_max_light_lines(1)
        .draw()?;

    // for (i, values) in data.iter().enumerate() {
    //     let color = *colors.get(i % colors.len()).unwrap_or(&BLACK);
    //     let label = labels.get(i % labels.len());
    //     let label = label.unwrap().as_str();
    //     chart
    //         .draw_series(LineSeries::new(
    //             values
    //                 .iter()
    //                 .map(|(date, x)| (NaiveDate::from_str(date).unwrap(), *x)),
    //             &color,
    //         ))?
    //         .label(label)
    //         .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
    // }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    // fn test_histogram() {
    //     histogram();
    // }

    #[test]
    fn response_time_set1() {
        let tasks = Tasks::load(&PathBuf::from("task_sets/task_set1.json")).unwrap();
        let response_time = tasks.response_time(false);
        println!("{}", response_time);
        plot(
            &PathBuf::from("plotters-doc-data/histogram.svg"),
            &response_time,
        )
        .unwrap();
    }
}
