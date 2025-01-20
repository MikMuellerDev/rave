<script lang="ts">
    import { Chart, Title, SubTitle, Tooltip, DoughnutController, ArcElement } from 'chart.js'
    import type { GaugeChartConfig } from './gaugeChart'
    import type { ChartConfiguration } from 'chart.js'
    import { onMount } from 'svelte'

    Chart.register(DoughnutController, ArcElement, Title, SubTitle, Tooltip)

    // Holds the chart's HTML canvas
    let chartCanvas: HTMLCanvasElement = undefined
    let chartCtx: CanvasRenderingContext2D = undefined
    let chart: Chart = undefined

    let options: ChartConfiguration = {
        type: 'doughnut',
        data: {
            labels: ['Consumed', 'Remaining'],
            datasets: [
                {
                    label: 'Whatever',
                    data: [0, 0],
                    backgroundColor: ['#000000', '#000000'],
                    borderColor: '#787878',
                    hoverOffset: 4,
                },
            ],
        },
        options: {
            rotation: 270, // start angle in degrees
            circumference: 180, // sweep angle in degrees
            responsive: true,
            // maintainAspectRatio: false,
            plugins: {
                title: {
                    display: true,
                    text: 'Loading...',
                    color: '#ffffff',
                    font: function (ctx) {
                        let width = ctx.chart.width;
                        let size = Math.round(width / 8);

                        return {
                            size,
                            weight: 'bold',
                        }
                    },
                    padding: {
                        top: 10,
                        bottom: -60,
                    },
                },
                subtitle: {
                    display: true,
                    text: 'Loading...',
                    color: '#9f9f9f',
                    font: function (ctx) {
                        let width = ctx.chart.width;
                        let size = Math.round(width / 10);

                        return {
                            size,
                        }
                    },
                    padding: {
                        top: 70,
                        bottom: 0,
                    },
                },
                tooltip: {
                    callbacks: {
                        label: (item: any) =>
                            ` ${item.raw} ${
                                item.raw === 1 ? config.unitSingular : config.unitPlural
                            }`,
                    },
                },
            },
            animation: {
                animateRotate: true,
                animateScale: true,
            },
        },
    }

    export let config: GaugeChartConfig = {
        eatenAmount: 0,
        totalAmount: 0,
        colorLessThanTotal: '00ff00',
        colorMoreThanTotal: 'ff0000',
        unitSingular: '',
        unitPlural: '',
    }
    $: updateInternal(config, chartCtx)

    function updateInternal(config: GaugeChartConfig, ctx: CanvasRenderingContext2D) {
        if (chart !== undefined) chart.data.datasets[0].data = calculateData(config)

        // Get RGB values from the primary color
        let rgb = hexToRgb(config.colorLessThanTotal)

        let fontColor = `#${config.colorLessThanTotal}`
        if (config.eatenAmount > config.totalAmount) {
            rgb = hexToRgb(config.colorMoreThanTotal)
            fontColor = `#${config.colorMoreThanTotal}`

            let diff = config.eatenAmount - config.totalAmount
            options.options.plugins.subtitle.text = `${diff} ${
                diff === 1 ? config.unitSingular : config.unitPlural
            } exceeded`
        } else {
            let diff = config.totalAmount - config.eatenAmount
            options.options.plugins.subtitle.text = `${diff} ${
                diff === 1 ? config.unitSingular : config.unitPlural
            } remaining`
        }

        options.options.plugins.title.color = fontColor
        options.options.plugins.title.text = `${config.eatenAmount} / ${config.totalAmount} ${
            config.eatenAmount == 1 ? config.unitSingular : config.unitPlural
        }`

        // Specify the gradient which is used to fill the area below the graph
        if (ctx !== undefined) {
            let gradient = ctx.createLinearGradient(0, 0, 600, 50)
            gradient.addColorStop(0.1, `rgb(${rgb[0]}, ${rgb[1]}, ${rgb[2]} )`)
            gradient.addColorStop(0.9, 'rgb(0, 0, 100, 0.125)')
            options.data.datasets[0].backgroundColor = [gradient, 'rgb(20, 20, 20)']
        }

        // options.data.datasets[0].backgroundColor = [
        //     `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]})`,
        //     '#000000',
        // ]

        if (chart !== undefined) chart.update()
    }

    function calculateData(config: GaugeChartConfig): number[] {
        if (config.eatenAmount > config.totalAmount) {
            return [config.eatenAmount, 0]
        } else {
            return [config.eatenAmount, config.totalAmount - config.eatenAmount]
        }
    }

    function hexToRgb(hex: string): [number, number, number] {
        var bigint = parseInt(hex, 16)
        var r = (bigint >> 16) & 255
        var g = (bigint >> 8) & 255
        var b = bigint & 255

        return [r, g, b]
    }

    onMount(() => {
        // Create a new canvas context
        chartCtx = chartCanvas.getContext('2d')

        updateInternal(config, chartCtx)

        chart = new Chart(chartCtx, options)
    })
</script>

<div class="container">
    <canvas bind:this={chartCanvas} class="container__chart" />
    <h6 class="text-disabled">{Math.round((config.eatenAmount / config.totalAmount) * 100)} %</h6>
</div>

<style lang="scss">
    @use '../../mixins' as *;

    .container {
        display: inline;
        width: 12rem;

        @include mobile {
            display: flex;
            flex-direction: column;
            align-items: center;
            width: 40%;
        }

        h6 {
            margin: 0 0;
        }
    }
</style>
