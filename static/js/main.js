// Overwrite loggers with the logging backend
window.onerror = function(msg, url, line) {
    window.external.invoke(JSON.stringify({
        Log: {
            kind: "error",
            msg: msg + " @ " + url + ":" + line
        }
    }));
};

// Borrowed from http://tobyho.com/2012/07/27/taking-over-console-log/
function intercept(method){
    console[method] = function(){
        var message = Array.prototype.slice.apply(arguments).join(' ');
        window.external.invoke(JSON.stringify({
            Log: {
                kind: method,
                msg: message
            }
        }));
    }
}

var methods = ['log', 'warn', 'error'];
for (var i = 0; i < methods.length; i++) {
    intercept(methods[i]);
}

document.getElementById("window-title").innerText = base_attributes.name + " Installer";

function selectFileCallback(name) {
    app.install_location = name;
}

var app = new Vue({
    router: router,
    data: {
        attrs: base_attributes,
        config : {},
        install_location : "",
        // If the option to pick an install location should be provided
        show_install_location : true,
        metadata : {
            database : [],
            install_path : "",
            preexisting_install : false
        }
    },
    methods: {
        "exit": function() {
            ajax("/api/exit", function() {});
        }
    }
}).$mount("#app");
