export function signup() {
    const response = await fetch("/signup/start");
    if (!response.ok) {
        throw new Error(`HTTP error! Status: ${response.status}`);
    }
    const json = await response.json();

    const passkey = await window.navigator.credentials.create({
        publicKey: PublicKeyCredential.parseRequestOptionsFromJSON(json);
    });

    const response = await fetch("/signup/finish", {
        method: "POST",
        body: passkey.toJSON(),
    });
    if (!response.ok) {
        throw new Error(`HTTP error! Status: ${response.status}`);
    }
    return await response.text();
}
