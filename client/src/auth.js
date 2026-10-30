export async function signup() {
    const response = await fetch("/api/signup/start");
    if (!response.ok) {
        throw new Error(`HTTP error! Status: ${response.status}`);
    }
    const json = await response.json();

    const passkey = await window.navigator.credentials.create({
        publicKey: PublicKeyCredential.parseRequestOptionsFromJSON(json),
    });

    const f_response = await fetch("/api/signup/finish", {
        method: "POST",
        body: passkey.toJSON(),
    });
    if (!f_response.ok) {
        throw new Error(`HTTP error! Status: ${response.status}`);
    }
    return await f_response.text();
}
