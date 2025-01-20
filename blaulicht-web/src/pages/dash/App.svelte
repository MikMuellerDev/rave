<script lang="ts">
    import { onMount } from 'svelte'
    import Page from '../../Page.svelte'
    import { loading } from '../../global'
    import { Button, Folder, List, Monitor, ThemeUtils, type ListOptions } from 'svelte-tweakpane-ui';
    import { Binding, type BindingObject } from 'svelte-tweakpane-ui';
    import { BlaulichtWebsocket, BlaulichtWebsocketCallbacks, topicAudioDevicesView, topicHeartbeat } from '../../lib/websocket';
    import { WaveformMonitor } from 'svelte-tweakpane-ui';

    async function loadAvailableAudioDevices(): Promise<String[]> {
        let res = (await fetch('/api/audio/devices')).json()
        console.log(res)
        return res
    }

    async function loadAvailableSerialDevices(): Promise<String[]> {
        let res = (await fetch('/api/serial/devices')).json()
        console.log(res)
        return res
    }

    //
    // Serial devices.
    //

    // Filled by web request.
    // let serialDevices = [""]

    let serialPortListOptions: ListOptions<string> = {};
    let selectedSerial = null

    // Filled by web request.
    // let audioDevices = [""]

    let audioPortListOptions: ListOptions<string> = {};
    $: console.dir(audioPortListOptions)
    let selectedAudio = null

    let socket: BlaulichtWebsocket | null = null

    onMount(async () => {
        $loading = true
        ThemeUtils.setGlobalDefaultTheme(ThemeUtils.presets.retro);

        // Connect socket.
        const callbacks = new BlaulichtWebsocketCallbacks()
        callbacks.subscribe(topicHeartbeat(), (event) => {
            console.log(`Heartbeat: ${event.value}`)
        })

        callbacks.subscribe(topicAudioDevicesView(), (event) => {
            const devices = event.value

            let audioPortListOptionsTemp = {}

            for (let dev of devices) {
                audioPortListOptionsTemp[dev] = dev
            }

            audioPortListOptions = audioPortListOptionsTemp
        })

        socket = new BlaulichtWebsocket(callbacks)

        // Serial devices.
        // for (let dev of serialDevices) {
        //     serialPortListOptions[dev] = dev
        // }

        // Audio devices.
        // for (let dev of audioDevices) {
        //     audioPortListOptions[dev] = dev
        // }

        // Waveform demo.
        setInterval(() => {
            waveData = waveData.map((v) =>
            Math.max(0, Math.min(10, v + (Math.random() * 2 - 1) * 0.5))
            );
        }, 50);

        setInterval(() => {
            numberToMonitor = Math.random() * 100;
        }, 50);

        $loading = false
    })

    let waveData = [5, 6, 7, 8, 9, 3, 9, 8, 7, 6, 5];
    let numberToMonitor = 85;

    async function selectAudio(device: string) {
        socket.send({
            kind: "SelectAudioDevice",
            value: device,
        })
    }

    async function selectSerial(device: string) {
        socket.send({
            kind: "SelectSerialDevice",
            value: device,
        })
    }
</script>

<Page pageId="dash">
    <div class="page">
        <div style="width: 100%; display: flex;">
            <div style="width: 50%;">
                <Button on:click={() => alert('🎛️')} />
            </div>

            <div style="width: 50%;">
                <Monitor value={numberToMonitor} graph={true} />
                <WaveformMonitor value={waveData} min={-1} max={11} lineStyle={'bezier'} />

                <Folder userExpandable={false} expanded={true} title="Devices">
                    <List
                        bind:value={selectedSerial}
                        label="Serial Port"
                        options={serialPortListOptions}
                        on:change={(e) => selectSerial(e.detail.value)}
                    />
                    <pre>Selected Option: {selectedSerial}</pre>

                    <List
                        bind:value={selectedAudio}
                        label="Audio Input" 
                        options={audioPortListOptions}
                        on:change={(e) => selectAudio(e.detail.value)}
                    />
                    <pre>Selected Option: {selectedAudio}</pre>
                </Folder>


                <!-- <Folder expanded={true} title="Reticulation Management Folder"> -->
                <!--     <Button on:click={() => console.log("incr")} title="Increment" /> -->
                <!--     <Monitor value={0} label="Count" /> -->
                <!-- </Folder> -->
            </div>
        </div>
    </div>
</Page>

<style lang="scss">
    @use '../../mixins' as *;
</style>
