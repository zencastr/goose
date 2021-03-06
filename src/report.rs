//! Optionally writes an html-formatted summary report after running a load test.

use crate::metrics;

use std::collections::BTreeMap;
use std::mem;

use chrono::prelude::*;
use serde::Serialize;
use serde_json::json;

/// The following templates are necessary to build an html-formatted summary report.
#[derive(Debug)]
pub struct GooseReportTemplates<'a> {
    pub raw_requests_template: &'a str,
    pub raw_responses_template: &'a str,
    pub co_requests_template: &'a str,
    pub co_responses_template: &'a str,
    pub tasks_template: &'a str,
    pub status_codes_template: &'a str,
    pub errors_template: &'a str,
    pub graph_rps_template: &'a str,
    pub graph_average_response_time_template: &'a str,
    pub graph_users_per_second: &'a str,
}

/// Defines the metrics reported about requests.
#[derive(Debug, Clone, Serialize)]
pub struct RequestMetric {
    pub method: String,
    pub name: String,
    pub number_of_requests: usize,
    pub number_of_failures: usize,
    pub response_time_average: String,
    pub response_time_minimum: usize,
    pub response_time_maximum: usize,
    pub requests_per_second: String,
    pub failures_per_second: String,
}

/// Defines the metrics reported about Coordinated Omission requests.
#[derive(Debug, Clone, Serialize)]
pub struct CORequestMetric {
    pub method: String,
    pub name: String,
    pub response_time_average: String,
    pub response_time_standard_deviation: String,
    pub response_time_maximum: usize,
}

/// Defines the metrics reported about responses.
#[derive(Debug, Clone, Serialize)]
pub struct ResponseMetric {
    pub method: String,
    pub name: String,
    pub percentile_50: String,
    pub percentile_60: String,
    pub percentile_70: String,
    pub percentile_80: String,
    pub percentile_90: String,
    pub percentile_95: String,
    pub percentile_99: String,
    pub percentile_100: String,
}

/// Defines the metrics reported about tasks.
#[derive(Debug, Clone, Serialize)]
pub struct TaskMetric {
    pub is_task_set: bool,
    pub task: String,
    pub name: String,
    pub number_of_requests: usize,
    pub number_of_failures: usize,
    pub response_time_average: String,
    pub response_time_minimum: usize,
    pub response_time_maximum: usize,
    pub requests_per_second: String,
    pub failures_per_second: String,
}

/// Defines the metrics reported about status codes.
pub struct StatusCodeMetric {
    pub method: String,
    pub name: String,
    pub status_codes: String,
}

/// Defines the HTML graph data.
#[derive(Debug)]
struct Graph<'a, T: Serialize> {
    pub html_id: &'a str,
    pub y_axis_label: &'a str,
    pub data: &'a [(String, T)],
    pub starting: Option<DateTime<Local>>,
    pub started: Option<DateTime<Local>>,
    pub stopping: Option<DateTime<Local>>,
    pub stopped: Option<DateTime<Local>>,
}

impl<'a, T: Serialize> Graph<'a, T> {
    /// Creates a new Graph object.
    fn new(
        html_id: &'a str,
        y_axis_label: &'a str,
        data: &'a [(String, T)],
        starting: Option<DateTime<Local>>,
        started: Option<DateTime<Local>>,
        stopping: Option<DateTime<Local>>,
        stopped: Option<DateTime<Local>>,
    ) -> Graph<'a, T> {
        Graph {
            html_id,
            y_axis_label,
            data,
            starting,
            started,
            stopping,
            stopped,
        }
    }

    /// Helper function to build HTML charts powered by the
    /// [ECharts](https://echarts.apache.org) library.
    fn generate_markup(self) -> String {
        let datetime_format = "%Y-%m-%d %H:%M:%S";

        let starting_area = if self.starting.is_some() && self.started.is_some() {
            format!(
                r#"[
                    {{
                        name: 'Starting',
                        xAxis: '{starting}'
                    }},
                    {{
                        xAxis: '{started}'
                    }}
                ],"#,
                starting = self.starting.unwrap().format(datetime_format),
                started = self.started.unwrap().format(datetime_format),
            )
        } else {
            "".to_string()
        };

        let stopping_area = if self.stopping.is_some() && self.stopped.is_some() {
            format!(
                r#"[
                    {{
                        name: 'Stopping',
                        xAxis: '{stopping}'
                    }},
                    {{
                        xAxis: '{stopped}'
                    }}
                ],"#,
                stopping = self.stopping.unwrap().format(datetime_format),
                stopped = self.stopped.unwrap().format(datetime_format),
            )
        } else {
            "".to_string()
        };

        format!(
            r#"<div class="graph">
                <div id="{html_id}" style="width: 1000px; height:500px; background: white;"></div>

                <script type="text/javascript">
                    var chartDom = document.getElementById('{html_id}');
                    var myChart = echarts.init(chartDom);

                    myChart.setOption({{
                        color: ['#2c664f'],
                        tooltip: {{ trigger: 'axis' }},
                        toolbox: {{
                            feature: {{
                                dataZoom: {{ yAxisIndex: 'none' }},
                                restore: {{}},
                                saveAsImage: {{}}
                            }}
                        }},
                        dataZoom: [
                            {{
                                type: 'inside',
                                start: 0,
                                end: 100,
                                fillerColor: 'rgba(34, 80, 61, 0.25)',
                                selectedDataBackground: {{
                                    lineStyle: {{ color: '#2c664f' }},
                                    areaStyle: {{ color: '#378063' }}
                                }}
                            }},
                            {{
                                start: 0,
                                end: 100,
                                fillerColor: 'rgba(34, 80, 61, 0.25)',
                                selectedDataBackground: {{
                                    lineStyle: {{ color: '#2c664f' }},
                                    areaStyle: {{ color: '#378063' }}
                                }}
                            }},
                        ],
                        xAxis: {{ type: 'time' }},
                        yAxis: {{
                            name: '{y_axis_label}',
                            nameLocation: 'center',
                            nameRotate: 90,
                            nameGap: 45,
                            type: 'value'
                        }},
                        series: [
                            {{
                                type: 'line',
                                symbol: 'none',
                                sampling: 'lttb',
                                lineStyle: {{ color: '#2c664f' }},
                                areaStyle: {{ color: '#378063' }},
                                markArea: {{
                                    itemStyle: {{ color: 'rgba(6, 6, 6, 0.10)' }},
                                    data: [
                                        {starting_area}
                                        {stopping_area}
                                    ]
                                }},
                                data: {values},
                            }}
                        ]
                    }});
                </script>
            </div>"#,
            html_id = self.html_id,
            values = json!(self.data),
            starting_area = starting_area,
            stopping_area = stopping_area,
            y_axis_label = self.y_axis_label,
        )
    }
}

/// Helper to generate a single response metric.
pub fn get_response_metric(
    method: &str,
    name: &str,
    response_times: &BTreeMap<usize, usize>,
    total_request_count: usize,
    response_time_minimum: usize,
    response_time_maximum: usize,
) -> ResponseMetric {
    // Calculate percentiles in a loop.
    let mut percentiles = Vec::new();
    for percent in &[0.5, 0.6, 0.7, 0.8, 0.9, 0.95, 0.99, 1.0] {
        percentiles.push(metrics::calculate_response_time_percentile(
            response_times,
            total_request_count,
            response_time_minimum,
            response_time_maximum,
            *percent,
        ));
    }

    // Now take the Strings out of the Vector and build a ResponseMetric object.
    ResponseMetric {
        method: method.to_string(),
        name: name.to_string(),
        percentile_50: mem::take(&mut percentiles[0]),
        percentile_60: mem::take(&mut percentiles[1]),
        percentile_70: mem::take(&mut percentiles[2]),
        percentile_80: mem::take(&mut percentiles[3]),
        percentile_90: mem::take(&mut percentiles[4]),
        percentile_95: mem::take(&mut percentiles[5]),
        percentile_99: mem::take(&mut percentiles[6]),
        percentile_100: mem::take(&mut percentiles[7]),
    }
}

/// Build an individual row of raw request metrics in the html report.
pub fn raw_request_metrics_row(metric: RequestMetric) -> String {
    format!(
        r#"<tr>
        <td>{method}</td>
        <td>{name}</td>
        <td>{number_of_requests}</td>
        <td>{number_of_failures}</td>
        <td>{response_time_average}</td>
        <td>{response_time_minimum}</td>
        <td>{response_time_maximum}</td>
        <td>{requests_per_second}</td>
        <td>{failures_per_second}</td>
    </tr>"#,
        method = metric.method,
        name = metric.name,
        number_of_requests = metric.number_of_requests,
        number_of_failures = metric.number_of_failures,
        response_time_average = metric.response_time_average,
        response_time_minimum = metric.response_time_minimum,
        response_time_maximum = metric.response_time_maximum,
        requests_per_second = metric.requests_per_second,
        failures_per_second = metric.failures_per_second,
    )
}

/// Build an individual row of response metrics in the html report.
pub fn response_metrics_row(metric: ResponseMetric) -> String {
    format!(
        r#"<tr>
            <td>{method}</td>
            <td>{name}</td>
            <td>{percentile_50}</td>
            <td>{percentile_60}</td>
            <td>{percentile_70}</td>
            <td>{percentile_80}</td>
            <td>{percentile_90}</td>
            <td>{percentile_95}</td>
            <td>{percentile_99}</td>
            <td>{percentile_100}</td>
        </tr>"#,
        method = metric.method,
        name = metric.name,
        percentile_50 = metric.percentile_50,
        percentile_60 = metric.percentile_60,
        percentile_70 = metric.percentile_70,
        percentile_80 = metric.percentile_80,
        percentile_90 = metric.percentile_90,
        percentile_95 = metric.percentile_95,
        percentile_99 = metric.percentile_99,
        percentile_100 = metric.percentile_100,
    )
}

/// If Coordinated Omission Mitigation is triggered, add a relevant request table to the
/// html report.
pub fn coordinated_omission_request_metrics_template(co_requests_rows: &str) -> String {
    format!(
        r#"<div class="CO requests">
        <h2>Request Metrics With Coordinated Omission Mitigation</h2>
        <table>
            <thead>
                <tr>
                    <th>Method</th>
                    <th>Name</th>
                    <th>Average (ms)</th>
                    <th>Standard deviation (ms)</th>
                    <th>Max (ms)</th>
                </tr>
            </thead>
            <tbody>
                {co_requests_rows}
            </tbody>
        </table>
    </div>"#,
        co_requests_rows = co_requests_rows,
    )
}

/// Build an individual row of Coordinated Omission Mitigation request metrics in
/// the html report.
pub fn coordinated_omission_request_metrics_row(metric: CORequestMetric) -> String {
    format!(
        r#"<tr>
            <td>{method}</td>
            <td>{name}</td>
            <td>{average})</td>
            <td>{standard_deviation}</td>
            <td>{maximum}</td>
        </tr>"#,
        method = metric.method,
        name = metric.name,
        average = metric.response_time_average,
        standard_deviation = metric.response_time_standard_deviation,
        maximum = metric.response_time_maximum,
    )
}

/// If Coordinated Omission Mitigation is triggered, add a relevant response table to the
/// html report.
pub fn coordinated_omission_response_metrics_template(co_responses_rows: &str) -> String {
    format!(
        r#"<div class="responses">
        <h2>Response Time Metrics With Coordinated Omission Mitigation</h2>
        <table>
            <thead>
                <tr>
                    <th>Method</th>
                    <th>Name</th>
                    <th>50%ile (ms)</th>
                    <th>60%ile (ms)</th>
                    <th>70%ile (ms)</th>
                    <th>80%ile (ms)</th>
                    <th>90%ile (ms)</th>
                    <th>95%ile (ms)</th>
                    <th>99%ile (ms)</th>
                    <th>100%ile (ms)</th>
                </tr>
            </thead>
            <tbody>
                {co_responses_rows}
            </tbody>
        </table>
    </div>"#,
        co_responses_rows = co_responses_rows,
    )
}

/// Build an individual row of Coordinated Omission Mitigation request metrics in
/// the html report.
pub fn coordinated_omission_response_metrics_row(metric: ResponseMetric) -> String {
    format!(
        r#"<tr>
            <td>{method}</td>
            <td>{name}</td>
            <td>{percentile_50}</td>
            <td>{percentile_60}</td>
            <td>{percentile_70}</td>
            <td>{percentile_80}</td>
            <td>{percentile_90}</td>
            <td>{percentile_95}</td>
            <td>{percentile_99}</td>
            <td>{percentile_100}</td>
        </tr>"#,
        method = metric.method,
        name = metric.name,
        percentile_50 = metric.percentile_50,
        percentile_60 = metric.percentile_60,
        percentile_70 = metric.percentile_70,
        percentile_80 = metric.percentile_80,
        percentile_90 = metric.percentile_90,
        percentile_95 = metric.percentile_95,
        percentile_99 = metric.percentile_99,
        percentile_100 = metric.percentile_100,
    )
}

/// If status code metrics are enabled, add a status code metrics table to the
/// html report.
pub fn status_code_metrics_template(status_code_rows: &str) -> String {
    format!(
        r#"<div class="status_codes">
        <h2>Status Code Metrics</h2>
        <table>
            <thead>
                <tr>
                    <th>Method</th>
                    <th colspan="2">Name</th>
                    <th colspan="3">Status Codes</th>
                </tr>
            </thead>
            <tbody>
                {status_code_rows}
            </tbody>
        </table>
    </div>"#,
        status_code_rows = status_code_rows,
    )
}

/// Build an individual row of status code metrics in the html report.
pub fn status_code_metrics_row(metric: StatusCodeMetric) -> String {
    format!(
        r#"<tr>
        <td>{method}</td>
        <td colspan="2">{name}</td>
        <td colspan="3">{status_codes}</td>
    </tr>"#,
        method = metric.method,
        name = metric.name,
        status_codes = metric.status_codes,
    )
}

/// If task metrics are enabled, add a task metrics table to the html report.
pub fn task_metrics_template(task_rows: &str, graph_tasks_per_second: &str) -> String {
    format!(
        r#"<div class="tasks">
        <h2>Task Metrics</h2>

        {graph_tasks_per_second}

        <table>
            <thead>
                <tr>
                    <th colspan="2">Task</th>
                    <th># Times Run</th>
                    <th># Fails</th>
                    <th>Average (ms)</th>
                    <th>Min (ms)</th>
                    <th>Max (ms)</th>
                    <th>RPS</th>
                    <th>Failures/s</th>
                </tr>
            </thead>
            <tbody>
                {task_rows}
            </tbody>
        </table>
    </div>"#,
        task_rows = task_rows,
        graph_tasks_per_second = graph_tasks_per_second,
    )
}

/// Build an individual row of task metrics in the html report.
pub fn task_metrics_row(metric: TaskMetric) -> String {
    if metric.is_task_set {
        format!(
            r#"<tr>
            <td colspan="10" align="left"><strong>{name}</strong></td>
        </tr>"#,
            name = metric.name,
        )
    } else {
        format!(
            r#"<tr>
            <td colspan="2">{task} {name}</strong></td>
            <td>{number_of_requests}</td>
            <td>{number_of_failures}</td>
            <td>{response_time_average}</td>
            <td>{response_time_minimum}</td>
            <td>{response_time_maximum}</td>
            <td>{requests_per_second}</td>
            <td>{failures_per_second}</td>
        </tr>"#,
            task = metric.task,
            name = metric.name,
            number_of_requests = metrics::format_number(metric.number_of_requests),
            number_of_failures = metrics::format_number(metric.number_of_failures),
            response_time_average = metric.response_time_average,
            response_time_minimum = metric.response_time_minimum,
            response_time_maximum = metric.response_time_maximum,
            requests_per_second = metric.requests_per_second,
            failures_per_second = metric.failures_per_second,
        )
    }
}

/// If there are errors, add an errors table to the html report.
pub fn errors_template(error_rows: &str, graph: &str) -> String {
    format!(
        r#"<div class="errors">
        <h2>Errors</h2>

        {graph}

        <table>
            <thead>
                <tr>
                    <th>#</th>
                    <th colspan="3">Error</th>
                </tr>
            </thead>
            <tbody>
                {error_rows}
            </tbody>
        </table>
    </div>"#,
        error_rows = error_rows,
        graph = graph,
    )
}

/// Build an individual error row in the html report.
pub fn error_row(error: &metrics::GooseErrorMetricAggregate) -> String {
    format!(
        r#"<tr>
        <td>{occurrences}</td>
        <td colspan="4">{error}</strong></td>
    </tr>"#,
        occurrences = error.occurrences,
        error = error.error,
    )
}

/// Build a requests per second graph.
pub fn graph_rps_template(
    rps: &[(String, u32)],
    starting: Option<DateTime<Local>>,
    started: Option<DateTime<Local>>,
    stopping: Option<DateTime<Local>>,
    stopped: Option<DateTime<Local>>,
) -> String {
    Graph::new(
        "graph-rps",
        "Requests #",
        rps,
        starting,
        started,
        stopping,
        stopped,
    )
    .generate_markup()
}

/// Build an errors per second graph.
pub fn graph_eps_template(
    eps: &[(String, u32)],
    starting: Option<DateTime<Local>>,
    started: Option<DateTime<Local>>,
    stopping: Option<DateTime<Local>>,
    stopped: Option<DateTime<Local>>,
) -> String {
    Graph::new(
        "graph-eps",
        "Errors #",
        eps,
        starting,
        started,
        stopping,
        stopped,
    )
    .generate_markup()
}

/// Build an average response time graph.
pub fn graph_average_response_time_template(
    response_times: &[(String, u32)],
    starting: Option<DateTime<Local>>,
    started: Option<DateTime<Local>>,
    stopping: Option<DateTime<Local>>,
    stopped: Option<DateTime<Local>>,
) -> String {
    Graph::new(
        "graph-avg-response-time",
        "Response time [ms]",
        response_times,
        starting,
        started,
        stopping,
        stopped,
    )
    .generate_markup()
}

/// Build a users per second graph.
pub fn graph_users_per_second_template(
    active_users: &[(String, usize)],
    starting: Option<DateTime<Local>>,
    started: Option<DateTime<Local>>,
    stopping: Option<DateTime<Local>>,
    stopped: Option<DateTime<Local>>,
) -> String {
    Graph::new(
        "graph-active-users",
        "Active users #",
        active_users,
        starting,
        started,
        stopping,
        stopped,
    )
    .generate_markup()
}

/// Build a tasks per second graph.
pub fn graph_tasks_per_second_template<T: Serialize>(
    tps: &[(String, T)],
    starting: Option<DateTime<Local>>,
    started: Option<DateTime<Local>>,
    stopping: Option<DateTime<Local>>,
    stopped: Option<DateTime<Local>>,
) -> String {
    Graph::new(
        "graph-tps",
        "Tasks #",
        tps,
        starting,
        started,
        stopping,
        stopped,
    )
    .generate_markup()
}

/// Build the html report.
pub fn build_report(
    users: &str,
    report_range: &str,
    hosts: &str,
    templates: GooseReportTemplates,
) -> String {
    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Goose Attack Report</title>
    <style>
        .container {{
            width: 1000px;
            margin: 0 auto;
            padding: 10px;
            background: #173529;
            font-family: Arial, Helvetica, sans-serif;
            font-size: 14px;
            color: #fff;
        }}

        .info span{{
            color: #b3c3bc;
        }}

        table {{
            border-collapse: collapse;
            text-align: center;
            width: 100%;
        }}

        td, th {{
            border: 1px solid #cad9ea;
            color: #666;
            height: 30px;
        }}

        thead th {{
            background-color: #cce8eb;
            width: 100px;
        }}

        tr:nth-child(odd) {{
            background: #fff;
        }}

        tr:nth-child(even) {{
            background: #f5fafa;
        }}

        .charts-container .chart {{
            width: 100%;
            height: 350px;
            margin-bottom: 30px;
        }}

        .download {{
            float: right;
        }}

        .download a {{
            color: #00ca5a;
        }}

        .graph {{
            margin-bottom: 1em;
        }}
    </style>
    <script src="https://cdn.jsdelivr.net/npm/echarts@5.2.2/dist/echarts.min.js"></script>
</head>
<body>
    <div class="container">
        <h1>Goose Attack Report</h1>

        <div class="info">
            <p>Users: <span>{users}</span> </p>
            <p>Target Host: <span>{hosts}</span></p>
            {report_range}
            <p><span><small><em>{pkg_name} v{pkg_version}</em></small></span></pr>
        </div>

        <div class="requests">
            <h2>Request Metrics</h2>

            {graph_rps_template}

            <table>
                <thead>
                    <tr>
                        <th>Method</th>
                        <th>Name</th>
                        <th># Requests</th>
                        <th># Fails</th>
                        <th>Average (ms)</th>
                        <th>Min (ms)</th>
                        <th>Max (ms)</th>
                        <th>RPS</th>
                        <th>Failures/s</th>
                    </tr>
                </thead>
                <tbody>
                    {raw_requests_template}
                </tbody>
            </table>
        </div>

        {co_requests_template}

        <div class="responses">
            <h2>Response Time Metrics</h2>

            {graph_average_response_time_template}

            <table>
                <thead>
                    <tr>
                        <th>Method</th>
                        <th>Name</th>
                        <th>50%ile (ms)</th>
                        <th>60%ile (ms)</th>
                        <th>70%ile (ms)</th>
                        <th>80%ile (ms)</th>
                        <th>90%ile (ms)</th>
                        <th>95%ile (ms)</th>
                        <th>99%ile (ms)</th>
                        <th>100%ile (ms)</th>
                    </tr>
                </thead>
                <tbody>
                    {raw_responses_template}
                </tbody>
            </table>
        </div>

        {co_responses_template}

        {status_codes_template}

        {tasks_template}

        <div class="users">
        <h2>User Metrics</h2>
            {graph_users_per_second}
        </div>

        {errors_template}

    </div>
</body>
</html>"#,
        users = users,
        report_range = report_range,
        hosts = hosts,
        pkg_name = pkg_name,
        pkg_version = pkg_version,
        raw_requests_template = templates.raw_requests_template,
        raw_responses_template = templates.raw_responses_template,
        co_requests_template = templates.co_requests_template,
        co_responses_template = templates.co_responses_template,
        tasks_template = templates.tasks_template,
        status_codes_template = templates.status_codes_template,
        errors_template = templates.errors_template,
        graph_rps_template = templates.graph_rps_template,
        graph_average_response_time_template = templates.graph_average_response_time_template,
        graph_users_per_second = templates.graph_users_per_second,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    fn expected_graph_html_prefix(html_id: &str, y_axis_label: &str) -> String {
        format!(
            r#"<div class="graph">
                <div id="{html_id}" style="width: 1000px; height:500px; background: white;"></div>

                <script type="text/javascript">
                    var chartDom = document.getElementById('{html_id}');
                    var myChart = echarts.init(chartDom);

                    myChart.setOption({{
                        color: ['#2c664f'],
                        tooltip: {{ trigger: 'axis' }},
                        toolbox: {{
                            feature: {{
                                dataZoom: {{ yAxisIndex: 'none' }},
                                restore: {{}},
                                saveAsImage: {{}}
                            }}
                        }},
                        dataZoom: [
                            {{
                                type: 'inside',
                                start: 0,
                                end: 100,
                                fillerColor: 'rgba(34, 80, 61, 0.25)',
                                selectedDataBackground: {{
                                    lineStyle: {{ color: '#2c664f' }},
                                    areaStyle: {{ color: '#378063' }}
                                }}
                            }},
                            {{
                                start: 0,
                                end: 100,
                                fillerColor: 'rgba(34, 80, 61, 0.25)',
                                selectedDataBackground: {{
                                    lineStyle: {{ color: '#2c664f' }},
                                    areaStyle: {{ color: '#378063' }}
                                }}
                            }},
                        ],
                        xAxis: {{ type: 'time' }},
                        yAxis: {{
                            name: '{y_axis_label}',
                            nameLocation: 'center',
                            nameRotate: 90,
                            nameGap: 45,
                            type: 'value'
                        }},
                        series: [
                            {{
                                type: 'line',
                                symbol: 'none',
                                sampling: 'lttb',
                                lineStyle: {{ color: '#2c664f' }},
                                areaStyle: {{ color: '#378063' }},
                                markArea: {{
                                    itemStyle: {{ color: 'rgba(6, 6, 6, 0.10)' }},
"#,
            html_id = html_id,
            y_axis_label = y_axis_label
        )
    }

    #[test]
    fn test_graph_rps_template() {
        let expected_prefix = expected_graph_html_prefix("graph-rps", "Requests #");

        let data = vec![
            ("2021-11-21 21:20:32".to_string(), 123),
            ("2021-11-21 21:20:33".to_string(), 111),
            ("2021-11-21 21:20:34".to_string(), 99),
            ("2021-11-21 21:20:35".to_string(), 134),
        ];

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        
                                        
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(graph_rps_template(&data, None, None, None, None), expected);

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        [
                    {
                        name: 'Starting',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                        
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_rps_template(
                &data,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34)),
                None,
                None
            ),
            expected
        );

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        
                                        [
                    {
                        name: 'Stopping',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_rps_template(
                &data,
                None,
                None,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34))
            ),
            expected
        );

        let mut expected = expected_prefix;
        expected.push_str(r#"                                    data: [
                                        [
                    {
                        name: 'Starting',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                        [
                    {
                        name: 'Stopping',
                        xAxis: '2021-11-21 21:20:36'
                    },
                    {
                        xAxis: '2021-11-21 21:20:38'
                    }
                ],
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_rps_template(
                &data,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 36)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 38))
            ),
            expected
        );
    }

    #[test]
    fn test_graph_eps_template() {
        let expected_prefix = expected_graph_html_prefix("graph-eps", "Errors #");

        let data = vec![
            ("2021-11-21 21:20:32".to_string(), 123),
            ("2021-11-21 21:20:33".to_string(), 111),
            ("2021-11-21 21:20:34".to_string(), 99),
            ("2021-11-21 21:20:35".to_string(), 134),
        ];

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        
                                        
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(graph_eps_template(&data, None, None, None, None), expected);

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        [
                    {
                        name: 'Starting',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                        
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_eps_template(
                &data,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34)),
                None,
                None
            ),
            expected
        );

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        
                                        [
                    {
                        name: 'Stopping',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_eps_template(
                &data,
                None,
                None,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34))
            ),
            expected
        );

        let mut expected = expected_prefix;
        expected.push_str(r#"                                    data: [
                                        [
                    {
                        name: 'Starting',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                        [
                    {
                        name: 'Stopping',
                        xAxis: '2021-11-21 21:20:36'
                    },
                    {
                        xAxis: '2021-11-21 21:20:38'
                    }
                ],
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_eps_template(
                &data,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 36)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 38))
            ),
            expected
        );
    }

    #[test]
    fn test_graph_average_response_time_template() {
        let expected_prefix =
            expected_graph_html_prefix("graph-avg-response-time", "Response time [ms]");

        let data = vec![
            ("2021-11-21 21:20:32".to_string(), 123),
            ("2021-11-21 21:20:33".to_string(), 111),
            ("2021-11-21 21:20:34".to_string(), 99),
            ("2021-11-21 21:20:35".to_string(), 134),
        ];

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        
                                        
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_average_response_time_template(&data, None, None, None, None),
            expected
        );

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        [
                    {
                        name: 'Starting',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                        
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_average_response_time_template(
                &data,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34)),
                None,
                None
            ),
            expected
        );

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        
                                        [
                    {
                        name: 'Stopping',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_average_response_time_template(
                &data,
                None,
                None,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34))
            ),
            expected
        );

        let mut expected = expected_prefix;
        expected.push_str(r#"                                    data: [
                                        [
                    {
                        name: 'Starting',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                        [
                    {
                        name: 'Stopping',
                        xAxis: '2021-11-21 21:20:36'
                    },
                    {
                        xAxis: '2021-11-21 21:20:38'
                    }
                ],
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_average_response_time_template(
                &data,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 36)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 38))
            ),
            expected
        );
    }

    #[test]
    fn test_graph_users_per_second_template() {
        let expected_prefix = expected_graph_html_prefix("graph-active-users", "Active users #");

        let data = vec![
            ("2021-11-21 21:20:32".to_string(), 123),
            ("2021-11-21 21:20:33".to_string(), 111),
            ("2021-11-21 21:20:34".to_string(), 99),
            ("2021-11-21 21:20:35".to_string(), 134),
        ];

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        
                                        
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_users_per_second_template(&data, None, None, None, None),
            expected
        );

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        [
                    {
                        name: 'Starting',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                        
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_users_per_second_template(
                &data,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34)),
                None,
                None
            ),
            expected
        );

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        
                                        [
                    {
                        name: 'Stopping',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_users_per_second_template(
                &data,
                None,
                None,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34))
            ),
            expected
        );

        let mut expected = expected_prefix;
        expected.push_str(r#"                                    data: [
                                        [
                    {
                        name: 'Starting',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                        [
                    {
                        name: 'Stopping',
                        xAxis: '2021-11-21 21:20:36'
                    },
                    {
                        xAxis: '2021-11-21 21:20:38'
                    }
                ],
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_users_per_second_template(
                &data,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 36)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 38))
            ),
            expected
        );
    }

    #[test]
    fn test_graph_tasks_per_second_template() {
        let expected_prefix = expected_graph_html_prefix("graph-tps", "Tasks #");

        let data = vec![
            ("2021-11-21 21:20:32".to_string(), 123),
            ("2021-11-21 21:20:33".to_string(), 111),
            ("2021-11-21 21:20:34".to_string(), 99),
            ("2021-11-21 21:20:35".to_string(), 134),
        ];

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        
                                        
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_tasks_per_second_template(&data, None, None, None, None),
            expected
        );

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        [
                    {
                        name: 'Starting',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                        
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_tasks_per_second_template(
                &data,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34)),
                None,
                None
            ),
            expected
        );

        let mut expected = expected_prefix.to_owned();
        expected.push_str(r#"                                    data: [
                                        
                                        [
                    {
                        name: 'Stopping',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_tasks_per_second_template(
                &data,
                None,
                None,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34))
            ),
            expected
        );

        let mut expected = expected_prefix;
        expected.push_str(r#"                                    data: [
                                        [
                    {
                        name: 'Starting',
                        xAxis: '2021-11-21 21:20:32'
                    },
                    {
                        xAxis: '2021-11-21 21:20:34'
                    }
                ],
                                        [
                    {
                        name: 'Stopping',
                        xAxis: '2021-11-21 21:20:36'
                    },
                    {
                        xAxis: '2021-11-21 21:20:38'
                    }
                ],
                                    ]
                                },
                                data: [["2021-11-21 21:20:32",123],["2021-11-21 21:20:33",111],["2021-11-21 21:20:34",99],["2021-11-21 21:20:35",134]],
                            }
                        ]
                    });
                </script>
            </div>"#
        );
        assert_eq!(
            graph_tasks_per_second_template(
                &data,
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 32)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 34)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 36)),
                Some(Local.ymd(2021, 11, 21).and_hms(21, 20, 38))
            ),
            expected
        );
    }
}
