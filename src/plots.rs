use crate::common::*;
use plotly::{
    common::{color::NamedColor, ErrorData, ErrorType},
    layout::{Axis, BarMode, Shape, ShapeLine, ShapeType},
    traces::Scatter,
    Bar, Layout, Plot,
};

use std::path::PathBuf;

fn plot(_path: &PathBuf, results: &TasksResult, show: bool) {
    let task_ids: Vec<String> = results.0.iter().map(|r| r.task.id.clone()).collect();
    let task_wcets: Vec<u32> = results.0.iter().map(|r| r.wcet).collect();
    let task_blockings: Vec<u32> = results.0.iter().map(|r| r.blocking).collect();
    let task_interferences: Vec<u32> = results
        .0
        .iter()
        .map(|r| r.interference.map_or_else(|| 0, |x| x))
        .collect();
    let task_deadlines: Vec<u32> = results.0.iter().map(|r| r.task.deadline).collect();

    let trace0 = Bar::new(task_ids.clone(), task_deadlines).name("Deadline");
    let trace1 = Bar::new(task_ids.clone(), task_wcets)
        .name("WCET")
        .legend_group("A");
    let trace2 = Bar::new(task_ids.clone(), task_blockings)
        .name("Block")
        .legend_group("A");
    let trace3 = Bar::new(task_ids.clone(), task_interferences)
        .name("Interference")
        .legend_group("A");
    // .error_y(
    //     ErrorData::new(ErrorType::Data)
    //         .array(vec![0.1, 0.2, 0.1, 0.1])
    //         .array_minus(vec![0.2, 0.4, 1., 0.2]),
    // );
    let layout = Layout::new().bar_mode(BarMode::Group);
    let mut plot = Plot::new();
    plot.set_layout(layout);
    plot.add_trace(trace0);
    let layout2 = Layout::new().bar_mode(BarMode::Stack);
    plot.set_layout(layout2);
    plot.add_traces(vec![trace1, trace2, trace3]);

    // let mut layout = Layout::new()
    //     .x_axis(Axis::new().range(vec![0.0, 7.0]).show_grid(false))
    //     .y_axis(Axis::new().range(vec![0.0, 3.5]));

    // layout.add_shape(
    //     Shape::new()
    //         .x_ref("x")
    //         .y_ref("y")
    //         .shape_type(ShapeType::Rect)
    //         .x0(1.)
    //         .y0(1.)
    //         .x1(2.)
    //         .y1(3.)
    //         .line(ShapeLine::new().color(NamedColor::RoyalBlue)),
    // );
    // layout.add_shape(
    //     Shape::new()
    //         .x_ref("x")
    //         .y_ref("y")
    //         .shape_type(ShapeType::Rect)
    //         .x0(3.)
    //         .y0(1.)
    //         .x1(6.)
    //         .y1(2.)
    //         .line(ShapeLine::new().color(NamedColor::RoyalBlue).width(2.))
    //         .fill_color(NamedColor::LightSkyBlue),
    // );

    // plot.set_layout(layout);
    if show {
        plot.show();
    }
    // println!("{}", plot.to_inline_html(Some("stacked_bar_chart")));
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
        let tasks = Tasks::load(&PathBuf::from("task_sets/task_set3.json")).unwrap();
        let response_time = tasks.response_time(false);
        println!("{}", response_time);
        plot(
            &PathBuf::from("plotters-doc-data/histogram.svg"),
            &response_time,
            true,
        );
    }
}
