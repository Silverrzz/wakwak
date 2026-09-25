if (self.name === "em-pthread" || self.name.startsWith("em-pthread-")) {
    importScripts(new URL("wakwak.js", self.location.href).href);
} else {
    const ready = (async () => {
        if (!self.crossOriginIsolated || typeof SharedArrayBuffer === "undefined") {
            throw new Error("Shared memory requires COOP: same-origin and COEP: require-corp headers over HTTPS or localhost");
        }
        const moduleUrl = new URL("wakwak.js", self.location.href).href;
        importScripts(moduleUrl);
        const engine = await createWakwak({
            locateFile: path => new URL(path, moduleUrl).href,
            print: line => postMessage(line),
            printErr: line => postMessage(line),
        });
        const deadline = performance.now() + 60000;
        while (!engine._wakwak_ready()) {
            if (performance.now() > deadline) throw new Error("Engine initialization timed out");
            await new Promise(resolve => setTimeout(resolve, 10));
        }
        return engine;
    })();

    ready.catch(error => postMessage(`info string Browser engine error: ${error.message}`));

    self.onmessage = async ({ data }) => {
        try {
            if (typeof data !== "string" || data.includes("\0")) throw new Error("Expected a UCI command string");
            const engine = await ready;
            for (const command of data.split(/\r?\n/).map(line => line.trim()).filter(Boolean)) {
                const result = engine.ccall("wakwak_command", "number", ["string"], [command]);
                if (result !== 0) throw new Error(`Command rejected (${result})`);
            }
        } catch (error) {
            postMessage(`info string Browser engine error: ${error.message}`);
        }
    };
}
