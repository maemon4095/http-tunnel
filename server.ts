const listener = Deno.listen({ port: 8080, });

for await (const conn of listener) {
    console.log("connected: ", conn);
    while (true) {
        const now = new Date();
        const iso = now.toISOString();
        const chunk = new TextEncoder().encode(`${iso}\n`);
        await conn.write(chunk);
        await wait(500);
    }
}

async function wait(ms: number) {
    await new Promise(resolve => {
        setTimeout(resolve, ms);
    });
}