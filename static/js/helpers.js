/**
 * helpers.js
 *
 * Additional state-less helper methods.
 */

var request_id = 0;

/**
 * Makes a AJAX request.
 *
 * @param path The path to connect to.
 * @param successCallback A callback with a JSON payload.
 * @param failCallback A fail callback. Optional.
 * @param data POST data. Optional.
 */
function ajax(path, successCallback, failCallback, data) {
    if (failCallback === undefined) {
        failCallback = defaultFailHandler;
    }

    var req = new XMLHttpRequest();

    req.addEventListener("load", function() {
        // The server can sometimes return a string error. Make sure we handle this.
        if (this.status === 200 && this.getResponseHeader('Content-Type').indexOf("application/json") !== -1) {
            successCallback(JSON.parse(this.responseText));
        } else {
            failCallback();
        }
    });
    req.addEventListener("error", failCallback);

    req.open(data == null ? "GET" : "POST", path + "?nocache=" + request_id++, true);
    // Rocket only currently supports URL encoded forms.
    req.setRequestHeader("Content-Type", "application/x-www-form-urlencoded");

    if (data != null) {
        var form = "";

        for (var key in data) {
            if (form !== "") {
                form += "&";
            }
            form += encodeURIComponent(key) + "=" + encodeURIComponent(data[key]);
        }

        req.send(form);
    } else {
        req.send();
    }
}

/**
 * The default handler if a AJAX request fails. Not to be used directly.
 *
 * @param e The XMLHttpRequest that failed.
 */
function defaultFailHandler(e) {
    console.error("A AJAX request failed, and was not caught:");
    console.error(e);
}
