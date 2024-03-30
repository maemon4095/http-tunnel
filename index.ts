const res = await fetch("http://serve.kmaeda.uk");

console.log(res);

if (!res.body) {
    Deno.exit();
}
const reader = res.body.pipeThrough(new TextDecoderStream()).getReader();
while (true) {
    const { done, value } = await reader.read();
    if (done) {
        break;
    }
    console.log(value);
}