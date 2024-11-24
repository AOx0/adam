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
 * @type {Array<{date: any, pass: number, blocked: number}>}
 */
const data = raw_data.map((d) => {
  const date = d3.timeParse("%Y-%m-%dT%H:%M:%S.%f")(d.time.substring(0, 26));
  const roundedDate = new Date(
    Math.floor(date.getTime() / (5 * 60 * 1000)) * (5 * 60 * 1000),
  );
  return {
    date: roundedDate,
    pass: d.event === "pass" ? 1 : 0,
    blocked: d.event.blocked ? 1 : 0,
  };
});

// Aggregate data by 5-minute intervals
let aggregatedData = d3
  .rollups(
    data,
    (v) => ({
      pass: d3.sum(v, (d) => d.pass),
      blocked: d3.sum(v, (d) => d.blocked),
    }),
    (d) => d.date,
  )
  .map(([date, values]) => ({ date, ...values }));

// Add X axis --> it is a date format
/**
 * @type {d3.ScaleTime<number, number>}
 */
const x = d3
  .scaleTime()
  .domain([
    d3.min(aggregatedData, (d) => d.date),
    new Date(d3.max(aggregatedData, (d) => d.date).getTime() + 5 * 60 * 1000),
  ])
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
  .domain([0, d3.max(aggregatedData, (d) => Math.max(d.pass, d.blocked)) * 1.2])
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
  .brushX()
  .extent([
    [0, 0],
    [width, height],
  ])
  .on("end", updateChart);

// Create the line plot variable: where both the line plot and the brush take place
const linePlot = svg.append("g").attr("clip-path", "url(#clip)");

// Add the lines
const linePass = d3
  .line()
  .defined(
    (d, i, data) =>
      d.pass !== null &&
      (i === 0 || d.date - data[i - 1].date === 5 * 60 * 1000),
  )
  .x((d) => x(d.date))
  .y((d) => y(d.pass));

const lineBlocked = d3
  .line()
  .defined(
    (d, i, data) =>
      d.blocked !== null &&
      (i === 0 || d.date - data[i - 1].date === 5 * 60 * 1000),
  )
  .x((d) => x(d.date))
  .y((d) => y(d.blocked));

// Add the area under the lines
const areaPass = d3
  .area()
  .defined(
    (d, i, data) =>
      d.pass !== null &&
      (i === 0 || d.date - data[i - 1].date === 5 * 60 * 1000),
  )
  .x((d) => x(d.date))
  .y0(y(0))
  .y1((d) => y(d.pass));

const areaBlocked = d3
  .area()
  .defined(
    (d, i, data) =>
      d.blocked !== null &&
      (i === 0 || d.date - data[i - 1].date === 5 * 60 * 1000),
  )
  .x((d) => x(d.date))
  .y0(y(0))
  .y1((d) => y(d.blocked));

linePlot
  .append("path")
  .datum(aggregatedData)
  .attr("class", "area pass")
  .attr("fill", "#69b3a2")
  .attr("opacity", 0.3)
  .attr("d", areaPass);

linePlot
  .append("path")
  .datum(aggregatedData)
  .attr("class", "area blocked")
  .attr("fill", "#ff6347")
  .attr("opacity", 0.3)
  .attr("d", areaBlocked);

linePlot
  .append("path")
  .datum(aggregatedData)
  .attr("class", "line pass")
  .attr("fill", "none")
  .attr("stroke", "#69b3a2")
  .attr("stroke-width", 1.5)
  .attr("d", linePass);

linePlot
  .append("path")
  .datum(aggregatedData)
  .attr("class", "line blocked")
  .attr("fill", "none")
  .attr("stroke", "#ff6347")
  .attr("stroke-width", 1.5)
  .attr("d", lineBlocked);

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
        "<br>Pass events: " +
        d.pass +
        "<br>Blocked events: " +
        d.blocked,
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
  .attr("cy", (d) => y(d.pass))
  .attr("r", 3)
  .attr("fill", "#69b3a2")
  .on("mouseover", showTooltip)
  .on("mousemove", moveTooltip)
  .on("mouseleave", hideTooltip);

const blockedCircles = linePlot
  .selectAll("circle.blocked")
  .data(aggregatedData)
  .enter()
  .append("circle")
  .attr("class", "blocked")
  .attr("cx", (d) => x(d.date))
  .attr("cy", (d) => y(d.blocked))
  .attr("r", 3)
  .attr("fill", "#ff6347")
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
    if (!idleTimeout) return (idleTimeout = setTimeout(idled, 350));
    x.domain([
      d3.min(aggregatedData, (d) => d.date),
      new Date(d3.max(aggregatedData, (d) => d.date).getTime() + 5 * 60 * 1000),
    ]);
    y.domain([
      0,
      d3.max(aggregatedData, (d) => Math.max(d.pass, d.blocked)) * 1.2,
    ]);
  } else {
    x.domain([x.invert(extent[0]), x.invert(extent[1])]);
    linePlot.select(".brush").call(brush.move, null); // This remove the grey brush area as soon as the selection has been done
  }

  // Update axis and line plot position
  xAxis.transition().duration(1000).call(d3.axisBottom(x));
  yAxis.transition().duration(1000).call(d3.axisLeft(y));
  linePlot.select(".line.pass").transition().duration(1000).attr("d", linePass);
  linePlot
    .select(".line.blocked")
    .transition()
    .duration(1000)
    .attr("d", lineBlocked);
  linePlot.select(".area.pass").transition().duration(1000).attr("d", areaPass);
  linePlot
    .select(".area.blocked")
    .transition()
    .duration(1000)
    .attr("d", areaBlocked);

  // Update all circles positions
  linePlot
    .selectAll("circle:not(.blocked)")
    .transition()
    .duration(1000)
    .attr("cx", (d) => x(d.date))
    .attr("cy", (d) => y(d.pass));

  linePlot
    .selectAll("circle.blocked")
    .transition()
    .duration(1000)
    .attr("cx", (d) => x(d.date))
    .attr("cy", (d) => y(d.blocked));
}

// If user double click, reinitialize the chart
svg.on("dblclick", function () {
  x.domain([
    d3.min(aggregatedData, (d) => d.date),
    new Date(d3.max(aggregatedData, (d) => d.date).getTime() + 5 * 60 * 1000),
  ]);
  y.domain([
    0,
    d3.max(aggregatedData, (d) => Math.max(d.pass, d.blocked)) * 1.2,
  ]);
  xAxis.transition().duration(1000).call(d3.axisBottom(x));
  yAxis.transition().duration(1000).call(d3.axisLeft(y));
  linePlot.select(".line.pass").transition().duration(1000).attr("d", linePass);
  linePlot
    .select(".line.blocked")
    .transition()
    .duration(1000)
    .attr("d", lineBlocked);
  linePlot.select(".area.pass").transition().duration(1000).attr("d", areaPass);
  linePlot
    .select(".area.blocked")
    .transition()
    .duration(1000)
    .attr("d", areaBlocked);

  // Update all circles
  linePlot
    .selectAll("circle:not(.blocked)")
    .transition()
    .duration(1000)
    .attr("cx", (d) => x(d.date))
    .attr("cy", (d) => y(d.pass));

  linePlot
    .selectAll("circle.blocked")
    .transition()
    .duration(1000)
    .attr("cx", (d) => x(d.date))
    .attr("cy", (d) => y(d.blocked));
});

const ws = new WebSocket(`ws://${ip}/firewall/events/ws`);
const liveUpdates = [];

ws.onmessage = (event) => {
  const parsedEvent = JSON.parse(event.data);

  if (!parsedEvent.kind || !parsedEvent.kind.event) {
    return;
  }

  const date = d3.timeParse("%Y-%m-%dT%H:%M:%S.%f")(
    parsedEvent.kind.event.time.substring(0, 26),
  );
  const roundedDate = new Date(
    Math.floor(date.getTime() / (5 * 60 * 1000)) * (5 * 60 * 1000),
  );
  liveUpdates.push({
    date: roundedDate,
    pass: parsedEvent.kind.event.event === "pass" ? 1 : 0,
    blocked: parsedEvent.kind.event.event.blocked ? 1 : 0,
  });
};

setInterval(() => {
  console.log("Live updates:", liveUpdates);
  if (liveUpdates.length === 0) return;

  data.push(...liveUpdates);
  liveUpdates.length = 0;

  // Re-aggregate data by 5-minute intervals
  aggregatedData = d3
    .rollups(
      data,
      (v) => ({
        pass: d3.sum(v, (d) => d.pass),
        blocked: d3.sum(v, (d) => d.blocked),
      }),
      (d) => d.date,
    )
    .map(([date, values]) => ({ date, ...values }));

  // Get current domains
  const currentXDomain = x.domain();

  // Only update Y domain if necessary
  const newYDomain = [
    0,
    d3.max(aggregatedData, (d) => Math.max(d.pass, d.blocked)) * 1.2,
  ];
  const needYUpdate = newYDomain[1] > y.domain()[1];

  if (needYUpdate) {
    y.domain(newYDomain);
    yAxis.transition().duration(1000).call(d3.axisLeft(y));
  }

  // Update line plots and areas
  linePlot
    .select(".line.pass")
    .datum(aggregatedData)
    .transition()
    .duration(1000)
    .attr("d", linePass);
  linePlot
    .select(".line.blocked")
    .datum(aggregatedData)
    .transition()
    .duration(1000)
    .attr("d", lineBlocked);
  linePlot
    .select(".area.pass")
    .datum(aggregatedData)
    .transition()
    .duration(1000)
    .attr("d", areaPass);
  linePlot
    .select(".area.blocked")
    .datum(aggregatedData)
    .transition()
    .duration(1000)
    .attr("d", areaBlocked);

  // Update and add new points for pass events
  const updatedCircles = linePlot
    .selectAll("circle:not(.blocked)")
    .data(aggregatedData);

  updatedCircles.exit().remove();

  const newCircles = updatedCircles
    .enter()
    .append("circle")
    .attr("r", 3)
    .attr("fill", "#69b3a2")
    .on("mouseover", showTooltip)
    .on("mousemove", moveTooltip)
    .on("mouseleave", hideTooltip);

  updatedCircles
    .merge(newCircles)
    .transition()
    .duration(1000)
    .attr("cx", (d) => x(d.date))
    .attr("cy", (d) => y(d.pass));

  // Update and add new points for blocked events
  const updatedBlockedCircles = linePlot
    .selectAll("circle.blocked")
    .data(aggregatedData);

  updatedBlockedCircles.exit().remove();

  const newBlockedCircles = updatedBlockedCircles
    .enter()
    .append("circle")
    .attr("class", "blocked")
    .attr("r", 3)
    .attr("fill", "#ff6347")
    .on("mouseover", showTooltip)
    .on("mousemove", moveTooltip)
    .on("mouseleave", hideTooltip);

  updatedBlockedCircles
    .merge(newBlockedCircles)
    .transition()
    .duration(1000)
    .attr("cx", (d) => x(d.date))
    .attr("cy", (d) => y(d.blocked));
}, 10000);
