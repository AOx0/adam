/**
 * @typedef {import('d3')} d3
 */

// set the dimensions and margins of the graph
const margin = { top: 10, right: 30, bottom: 30, left: 60 };

const width =
  document.getElementById("chart-container").offsetWidth -
  margin.left -
  margin.right;
const height = 300 - margin.top - margin.bottom;

// append the svg object to the body of the page
/**
 * @type {d3.Selection}
 */
const svg = d3
  .select("#chart-container")
  .append("svg")
  .attr("width", width + margin.left + margin.right)
  .attr("height", height + margin.top + margin.bottom)
  .append("g")
  .attr("transform", `translate(${margin.left},${margin.top})`);

/**
 * @type {Array<{date: any, value: number}>}
 */
const data = raw_data.map((d) => {
  const date = d3.timeParse("%Y-%m-%dT%H:%M:%S.%f")(d.time.substring(0, 26));
  const roundedDate = new Date(
    Math.floor(date.getTime() / (15 * 60 * 1000)) * (15 * 60 * 1000),
  );
  return {
    date: roundedDate,
    value: 1,
  };
});

// Aggregate data by 15-minute intervals
const aggregatedData = d3
  .rollups(
    data,
    (v) => d3.sum(v, (d) => d.value),
    (d) => d.date,
  )
  .map(([date, value]) => ({ date, value }));

// Add X axis --> it is a date format
/**
 * @type {d3.ScaleTime<number, number>}
 */
const x = d3
  .scaleTime()
  .domain(d3.extent(aggregatedData, (d) => d.date))
  .range([0, width]);
xAxis = svg
  .append("g")
  .attr("transform", `translate(0,${height})`)
  .call(d3.axisBottom(x));

// Add Y axis
/**
 * @type {d3.ScaleLinear<number, number>}
 */
const y = d3
  .scaleLinear()
  .domain([0, d3.max(aggregatedData, (d) => +d.value) + 1])
  .range([height, 0]);
yAxis = svg.append("g").call(d3.axisLeft(y));

// Add a clipPath: everything out of this area won't be drawn.
const clip = svg
  .append("defs")
  .append("clipPath")
  .attr("id", "clip")
  .append("rect")
  .attr("width", width)
  .attr("height", height)
  .attr("x", 0)
  .attr("y", 0);

// Add brushing
const brush = d3
  .brushX() // Add the brush feature using the d3.brush function
  .extent([
    [0, 0],
    [width, height],
  ]) // initialise the brush area: start at 0,0 and finishes at width,height: it means I select the whole graph area
  .on("end", updateChart); // Each time the brush selection changes, trigger the 'updateChart' function

// Create the line plot variable: where both the line plot and the brush take place
const linePlot = svg.append("g").attr("clip-path", "url(#clip)");

// Add the line
const line = d3
  .line()
  .x((d) => x(d.date))
  .y((d) => y(d.value));

linePlot
  .append("path")
  .datum(aggregatedData)
  .attr("class", "line")
  .attr("fill", "none")
  .attr("stroke", "#69b3a2")
  .attr("stroke-width", 1.5)
  .attr("d", line);

// Add the brushing
linePlot.append("g").attr("class", "brush").call(brush);

// Add tooltip
const tooltip = d3
  .select("#chart-container")
  .append("div")
  .style("opacity", 0)
  .attr(
    "class",
    "tooltip bg-background text-foreground p-2 rounded border border-foreground absolute",
  );

// Tooltip functions
const showTooltip = function (event, d) {
  tooltip.transition().duration(200).style("opacity", 1);
  tooltip
    .html(
      "Time: " +
        d3.timeFormat("%Y-%m-%d %H:%M:%S")(d.date) +
        "<br>Number of events: " +
        d.value,
    )
    .style("left", event.pageX + "px")
    .style("top", event.pageY + "px");
};

const moveTooltip = function (event) {
  tooltip.style("left", event.pageX + "px").style("top", event.pageY + "px");
};

const hideTooltip = function () {
  tooltip.transition().duration(200).style("opacity", 0);
};

// Add the line plot with tooltip
const circles = linePlot
  .selectAll("circle")
  .data(aggregatedData)
  .enter()
  .append("circle")
  .attr("cx", (d) => x(d.date))
  .attr("cy", (d) => y(d.value))
  .attr("r", 3)
  .attr("fill", "#69b3a2")
  .on("mouseover", showTooltip)
  .on("mousemove", moveTooltip)
  .on("mouseleave", hideTooltip);

// A function that set idleTimeOut to null
let idleTimeout;
function idled() {
  idleTimeout = null;
}

// A function that update the chart for given boundaries
function updateChart(event) {
  // What are the selected boundaries?
  extent = event.selection;

  // If no selection, back to initial coordinate. Otherwise, update X axis domain
  if (!extent) {
    if (!idleTimeout) return (idleTimeout = setTimeout(idled, 350)); // This allows to wait a little bit
    x.domain([4, 8]);
  } else {
    x.domain([x.invert(extent[0]), x.invert(extent[1])]);
    linePlot.select(".brush").call(brush.move, null); // This remove the grey brush area as soon as the selection has been done
  }

  // Update axis and line plot position
  xAxis.transition().duration(1000).call(d3.axisBottom(x));
  linePlot
    .select(".line")
    .transition()
    .duration(1000)
    .attr("d", line(aggregatedData));

  // Update scatter points position
  circles
    .transition()
    .duration(1000)
    .attr("cx", (d) => x(d.date))
    .attr("cy", (d) => y(d.value));
}

// If user double click, reinitialize the chart
svg.on("dblclick", function () {
  x.domain(d3.extent(aggregatedData, (d) => d.date));
  xAxis.transition().call(d3.axisBottom(x));
  linePlot.select(".line").transition().attr("d", line(aggregatedData));
  circles
    .transition()
    .attr("cx", (d) => x(d.date))
    .attr("cy", (d) => y(d.value));
});
