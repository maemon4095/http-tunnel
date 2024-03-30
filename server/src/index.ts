import { parseArgs } from "https://deno.land/std@0.221.0/cli/mod.ts";

const { connection, tunnel } = parseInput();
console.log("waiting for tunnel connection");
Deno.serve(tunnel, async (req) => {
    console.log("tunnel connected.");
    console.log("start connect to", connection);
    const conn = await Deno.connect(connection);
    req.body?.pipeTo(conn.writable);
    return new Response(conn.readable);
});

function parseInput() {
    const args = parseArgs(Deno.args, {
        string: ["--over"]
    });

    const tunnelAddress = args["over"] as string | number | undefined;
    if (!tunnelAddress) {
        throw new Error("--over option is missing.");
    }

    let tunhost;
    let tunport;
    if (typeof tunnelAddress === "number") {
        tunhost = "127.0.0.1";
        tunport = tunnelAddress;
    } else {
        const [host, tunportStr] = tunnelAddress.split(":", 2);
        tunport = tunportStr ? Number.parseInt(tunportStr) : 8000;
        tunhost = host;
    }

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
        const [host, conportStr] = connectionAddress.split(":", 2);
        conhost = host;
        conport = conportStr ? Number.parseInt(conportStr) : 80;
    }

    return {
        connection: {
            hostname: conhost,
            port: conport
        },
        tunnel: { hostname: tunhost, port: tunport }
    };
}