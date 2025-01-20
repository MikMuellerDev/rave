<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte'
    import type { DayReport, DayReportGeneric } from './meal'
    import 'chartjs-adapter-date-fns'

    let dispatch = createEventDispatcher()

    import {
        Chart,
        Title,
        Tooltip,
        LineElement,
        LinearScale,
        PointElement,
        CategoryScale,
        LineController,
        TimeScale,
        BarController,
        BarElement,
        Filler,
    } from 'chart.js'

    import type { ChartConfiguration } from 'chart.js'

    // Register all chart modules
    Chart.register(
        Title,
        Tooltip,
        Filler,
        LineElement,
        LineController,
        BarController,
        BarElement,
        LinearScale,
        PointElement,
        CategoryScale,
        TimeScale,
    )

    // Holds the chart's HTML canvas
    let chartCanvas: HTMLCanvasElement = undefined

    export let style = {
        backgroundColor: "#ff0000",
        lineTension: 0.0,
        borderColor: '#26a69a',
        pointBorderColor: '#26a69a',
        pointBackgroundColor: '#323232',
    }

    export let targetValue = 100

    export let label = ""
    export let categoryData = []

    $: updateInternal(categoryData)

    let chart: Chart = undefined

    let renderData = []


    function calculateData(rawData: DayReportGeneric[]) {
        renderData = []

        for (let record of rawData) {
            let d = new Date(record.date)

            renderData.push({
                x: d,
                y: record.value,
            })
        }
    }

    function updateInternal(data: DayReportGeneric[]) {
        calculateData(data)

        if (chart === undefined) return

        chart.data.datasets[0].data = renderData
        chart.data.datasets[1].data = renderData.length > 0 ? [{x: renderData[0].x, y: targetValue}, {x: renderData[renderData.length - 1].x, y: targetValue}] : []

        chart.update()
    }

    function hexToRgb(hex: string): [number, number, number] {
        var bigint = parseInt(hex, 16)
        var r = (bigint >> 16) & 255
        var g = (bigint >> 8) & 255
        var b = bigint & 255

        return [r, g, b]
    }

    const options: ChartConfiguration = {
        responsive: true,
        maintainAspectRatio: false,
        interaction: {
            mode: 'index',
            intersect: false,
        },
        stacked: false,
        plugins: {
            // @ts-ignore
            legend: {
                display: false,
            },
            tooltip: {
                callbacks: {
                    label: (item: any) => {
                        switch (item.dataset.label) {
                            case 'calories':
                                return `${item.parsed.y} kcal`
                            case 'protein':
                                return `${item.parsed.y} g protein`
                            case 'carbs':
                                return `${item.parsed.y} g carbs`
                            case 'fats':
                                return `${item.parsed.y} g fats`
                            case 'water':
                                return `${item.parsed.y} ml water`
                        }
                    },
                },
            },
        },
        scales: {
            y: {
                type: 'linear',
                display: true,
                position: 'left',
                suggestedMin: 0,
                suggestedMax: 1000
            },
            // y1: {
            //     type: 'linear',
            //     display: true,
            //     position: 'right',
            // },
            x: {
                type: 'time',
                beginAtZero: true,
                time: {
                    unit: 'day',
                },
            },
        },
        grid: {
            drawOnChartArea: false,
        },
        onClick: (_: Event, elements: any, chart: any) => {
            if (elements[0]) {
                const index = elements[0].index
                const date = chart.data.datasets[0].data[index].x
                dispatch('select', date)
            }
        },
    }

    onMount(async () => {
        // Create a new canvas context
        let ctx = chartCanvas.getContext('2d')

        function calculateGradient(color: string): CanvasGradient {
            // Get  RGB values from user's primary color
            const rgb = hexToRgb(color)

            // Specify the gradient which is used to fill the area below the graph
            let gradient = ctx.createLinearGradient(0, 0, 0, 400)
            gradient.addColorStop(0.125, `rgb(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, 0.3)`)
            gradient.addColorStop(0.75, 'rgb(0, 0, 100, 0.125)')
            //gradient.addColorStop(0.7, "rgb(0,  0, 100, 0.03)");
            return gradient
        }

        // Dataset configuration
        let datasets = {
            datasets: [
                {
                    label,
                    // The area below the graph should be filled
                    fill: true,
                    // Use the gradient as the background color
                    backgroundColor: calculateGradient(`#${style.backgroundColor}`),
                    // TODO: tweak this value to make it look the best
                    lineTension: style.lineTension,
                    // Make the graph's line appear in the user's primary color
                    borderColor: `#${style.borderColor}`,
                    // Make the point border color appear in the user's primary color
                    pointBorderColor: `#${style.pointBorderColor}`,
                    // The inner color of each point
                    pointBackgroundColor: `#${style.pointBackgroundColor}`,
                    // How wide the point's border should be
                    pointBorderWidth: 1,
                    // Sets the point's default width
                    pointRadius: 4,
                    // Threshold on which `hover` is triggered
                    pointHitRadius: 10,
                    // Widen the point on hover
                    pointHoverRadius: 6,
                    // Darken the point's inner color on hover
                    pointHoverBackgroundColor: '#222',
                    // Increment the point's border width on hover
                    pointHoverBorderWidth: 2,
                    // Could be used to make the graph's line dashed
                    //borderDash: [4],
                    data: renderData,
                    yAxisID: 'y',
                },
                {
                    label: "Target",
                    // The area below the graph should be filled
                    fill: false,
                    // Use the gradient as the background color
                    backgroundColor: "gray",
                    // TODO: tweak this value to make it look the best
                    lineTension: style.lineTension,
                    // Make the graph's line appear in the user's primary color
                    borderColor: 'rgba(255,255,255,0.1)',
                    // Make the point border color appear in the user's primary color
                    pointBorderColor: 'black',
                    // The inner color of each point
                    pointBackgroundColor: 'gray',
                    // How wide the point's border should be
                    pointBorderWidth: 0,
                    // Sets the point's default width
                    pointRadius: 0,
                    // Threshold on which `hover` is triggered
                    pointHitRadius: 0,
                    // Widen the point on hover
                    pointHoverRadius: 0,
                    // Darken the point's inner color on hover
                    pointHoverBackgroundColor: '#222',
                    // Increment the point's border width on hover
                    pointHoverBorderWidth: 0,
                    // Could be used to make the graph's line dashed
                    borderDash: [10, 5],
                    // data: renderData.map(({x, _}) => Object.create({x, y: targetValue})),
                    data: (renderData.length > 0) ? [{x: renderData[0].x, y: targetValue}, {x: renderData[renderData.length - 1].x, y: targetValue}] : [],
                    yAxisID: 'y',
                },
                // {
                //     label: 'protein',
                //     // The area below the graph should be filled
                //     fill: true,
                //     // Use the gradient as the background color
                //     backgroundColor: calculateGradient('#7E57C2'),
                //     // TODO: tweak this value to make it look the best
                //     lineTension: 0.0,
                //     // Make the graph's line appear in the user's primary color
                //     borderColor: '#7E57C2',
                //     // Make the point border color appear in the user's primary color
                //     pointBorderColor: '#7E57C2',
                //     // The inner color of each point
                //     pointBackgroundColor: '#323232',
                //     // How wide the point's border should be
                //     pointBorderWidth: 1,
                //     // Sets the point's default width
                //     pointRadius: 4,
                //     // Threshold on which `hover` is triggered
                //     pointHitRadius: 10,
                //     // Widen the point on hover
                //     pointHoverRadius: 6,
                //     // Darken the point's inner color on hover
                //     pointHoverBackgroundColor: '#222',
                //     // Increment the point's border width on hover
                //     pointHoverBorderWidth: 2,
                //     // Could be used to make the graph's line dashed
                //     //borderDash: [4],
                //     data: dataProtein,
                //     yAxisID: 'y1',
                // },
                // {
                //     label: 'carbs',
                //     // The area below the graph should be filled
                //     fill: true,
                //     // Use the gradient as the background color
                //     backgroundColor: calculateGradient('26a69a'),
                //     // TODO: tweak this value to make it look the best
                //     lineTension: 0.0,
                //     // Make the graph's line appear in the user's primary color
                //     borderColor: '#28DE5F',
                //     // Make the point border color appear in the user's primary color
                //     pointBorderColor: '#26a69a',
                //     // The inner color of each point
                //     pointBackgroundColor: '#323232',
                //     // How wide the point's border should be
                //     pointBorderWidth: 1,
                //     // Sets the point's default width
                //     pointRadius: 4,
                //     // Threshold on which `hover` is triggered
                //     pointHitRadius: 10,
                //     // Widen the point on hover
                //     pointHoverRadius: 6,
                //     // Darken the point's inner color on hover
                //     pointHoverBackgroundColor: '#222',
                //     // Increment the point's border width on hover
                //     pointHoverBorderWidth: 2,
                //     // Could be used to make the graph's line dashed
                //     //borderDash: [4],
                //     data: dataCarbs,
                //     yAxisID: 'y',
                // },
            ],
        }

        // Create the chart
        chart = new Chart(ctx, {
            type: 'line',
            data: datasets,
            // @ts-ignore
            options,
        })

        // Darkmode-specific configuration
        const x = chart.config.options.scales.x
        const y = chart.config.options.scales.y

        // Decrease the X and Y axis' grid opacity
        x.grid.color = 'rgba(255, 255, 255, 0.025)'
        y.grid.color = 'rgba(100, 100, 100, 0.25)'
        // Darken the tooltip background color
        Chart.defaults.plugins.tooltip.backgroundColor = 'rgba(50, 50, 50, 0.75)'
        // Remove borders around the X and Y axis
        x.grid.borderColor = 'transparent'
        y.grid.borderColor = 'transparent'
    })
</script>

<div class="chart-outer">
    <canvas bind:this={chartCanvas} class="chart" />
</div>

<style lang="scss">
    @use '../../mixins' as *;

    .chart-outer {
        height: 17rem;

        @include not-widescreen {
            height: calc(100vh / 100 * 14);
        }
    }

    // Chart styling
    .chart {
        background-color: transparent;
    }
</style>
