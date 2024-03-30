import { parseArgs } from "https://deno.land/std@0.221.0/cli/mod.ts";

const { connection, tunnel } = parseInput();
const listener = Deno.listen(connection);

console.log("waiting for connection.");
for await (const conn of listener) {
    console.log("connected.");
    console.log("open tunnel");
    const res = await fetch(tunnel, {
        method: "POST",
        body: conn.readable
    });
    if (res.body === null) {
        Deno.exit();
    }
    await res.body.pipeTo(conn.writable);
    console.log("connection closed.");
}

function parseInput() {
    const args = parseArgs(Deno.args, {
        string: ["--over"]
    });

    const tunnelUrlStr = args["over"] as string | number | undefined;
    if (!tunnelUrlStr) {
        throw new Error("--over option is missing.");
    }
    const tunnel = new URL(tunnelUrlStr.toString());

    const connectionAddress = args._[0];
    if (!connectionAddress) {
        throw new Error("positional argument [connection] is missing.");
    }
    let conhost;
    let conport;
    if (typeof connectionAddress === "number") {
        conhost = "127.0.0.1";
        conport = connectionAddress;
    } else {
        const [host, conportStr] = connectionAddress.toString().split(":", 2);
        conhost = host;
        conport = conportStr ? Number.parseInt(conportStr) : 80;
    }

    return {
        connection: {
            hostname: conhost,
            port: conport
        },
        tunnel
    };
}