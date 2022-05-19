let address = Deno.core.opSync("getAddress");
let config = Deno.core.opSync("getConfig");

console.log(address);
console.log(config.function);

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

(async () => {
    for (;;) {
        await sleep(250);
        try {
            console.log("sending request...");
            let resp = await fetch(`https://parallel.life/faucet/ticket/?eth-address=${address}`, {
                headers: {
                    'cookie': config.function,
                }
            });
            if (resp.status !== 200) {
                console.log("invalid response status:", resp.status);
                continue;
            }
            let data = await resp.json();
            console.log("resp:", data);
            if (!data.ticket || !data.card_id) {
                console.log("invalid response, trying again");
                continue;
            }
            console.log(data.ticket);
            console.log(data.card_id);
            Deno.core.opSync("returnTxData", { raw: data.ticket });
            break;
        } catch (e) {
            console.log("error sending req:", e);
        }
    }
})()
