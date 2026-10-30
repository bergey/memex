export async function signup() {
    const response = await fetch("/api/signup/start");
    if (!response.ok) {
        throw new Error(`HTTP error! Status: ${response.status}`);
    }
    const json = await response.json();
    json.publicKey.challenge = Uint8Array.fromBase64(json.publicKey.challenge, {alphabet: "base64url"});
    json.publicKey.user.id = Uint8Array.fromBase64(json.publicKey.user.id, {alphabet: "base64url"});

    const passkey = await window.navigator.credentials.create(json);

    const f_response = await fetch("/api/signup/finish", {
        method: "POST",
      body: JSON.stringify(passkey),
        headers: new Headers([["content-type", "application/json"]]),
    });
    if (!f_response.ok) {
        throw new Error(`HTTP error! Status: ${response.status}`);
    }
    return await f_response.text();
}
