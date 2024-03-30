const conn = await Deno.connect({ port: 3030 });
console.log(conn);

const body = conn.readable.pipeThrough(new TextDecoderStream());
const reader = body.getReader();

while (true) {
    const { done, value } = await reader.read();
    if (done) {
        break;
    }
    console.log(value);
}
reader.releaseLock();
console.log("done");

