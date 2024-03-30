const conn = await Deno.connect({ hostname: "localhost", port: 3030 });
console.log(conn);

const body = conn.readable.getReader();

while (true) {
    const { done, value } = await body.read();
    if (done) {
        break;
    }
    console.log(value);
}
body.releaseLock();
console.log("done");

