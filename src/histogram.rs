use crate::common::*;
use plotters::{
    coord::ranged1d::{DiscreteRanged, KeyPointHint, NoDefaultFormatting, ValueFormatter},
    prelude::*,
};
use std::path::PathBuf;

#[derive(Clone)]
struct CustomizedX((u32, Vec<String>));

impl Ranged for CustomizedX {
    type ValueType = u32;
    type FormatOption = NoDefaultFormatting;

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        let size = limit.1 - limit.0;
        ((*value as f64 / self.0 .0 as f64) * size as f64) as i32 + limit.0
    }

    fn range(&self) -> std::ops::Range<Self::ValueType> {
        0..self.0 .0
    }

    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<Self::ValueType> {
        if hint.max_num_points() < (self.0 .0 as usize) {
            return vec![];
        }

        (0..self.0 .0).collect()
    }
}

impl DiscreteRanged for CustomizedX {
    // Required methods
    fn size(&self) -> usize {
        self.0 .0 as usize
    }

    fn index_of(&self, value: &Self::ValueType) -> Option<usize> {
        if *value < self.size() as u32 {
            Some(*value as usize)
        } else {
            None
        }
    }

    fn from_index(&self, index: usize) -> Option<Self::ValueType> {
        if index < self.size() {
            Some(index as u32)
        } else {
            None
        }
    }
}

impl ValueFormatter<u32> for CustomizedX {
    fn format_ext(&self, value: &u32) -> String {
        println!("value {}", value);
        format!("{} of {}", value, self.0 .1.get(*value as usize).unwrap())
    }
}

fn histogram(path: &PathBuf, results: &TasksResult) -> Result<(), Box<dyn std::error::Error>> {
    let root = SVGBackend::new(path, (640, 480)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        // .baseline_func(|x| {
        //     if let SegmentValue::Exact(v) = x {
        //         *v
        //     } else {
        //         0
        //     }
        // })
        .caption("Response Time per Task", ("sans-serif", 50.0))
        // .build_cartesian_2d((0u32..10u32).into_segmented(), 0u32..10u32)?;
        .build_cartesian_2d(
            CustomizedX((
                7,
                vec![
                    "a".to_string(),
                    "b".to_string(),
                    "c".to_string(),
                    "d".to_string(),
                    "e".to_string(),
                    "f".to_string(),
                    "g".to_string(),
                ],
            ))
            .into_segmented(),
            0u32..10u32,
        )?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Response Time")
        .x_desc("Task")
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    let data = [1u32, 1, 1, 1, 4, 2, 5];

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(data.iter().map(|x: &u32| (*x, 1))),
        // .style(BLUE.mix(0.5).filled())
        // .data(data.iter().map(|x: &u32| (*x / 2, 1))),
    )?;

    // chart.draw_series(
    //     Histogram::vertical(&chart)
    //         .style(BLUE.mix(0.5).filled())
    //         .data(data.iter().map(|x: &u32| (*x / 2, 1))),
    // )?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {:?}", path);

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
        histogram(
            &PathBuf::from("plotters-doc-data/histogram.svg"),
            &response_time,
        )
        .unwrap();
    }
}
