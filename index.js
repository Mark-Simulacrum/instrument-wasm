const js = import("./instrument_wasm");
const memory = import("./instrument_wasm_bg");

js.then(js => {
    memory.then(memory => {
    let { Universe, Point } = js;

    const CELL_SIZE = 20;

    // Give the canvas room for all of our cells and a 1px border
    // around each of them.
    const canvas = document.getElementById("rust-canvas");
    // Construct the universe, and get its width and height.
    let universe = js.Universe.new(new window.AudioContext(), canvas.getContext('2d'));

    canvas.height = document.querySelector("body").getBoundingClientRect().height;
    canvas.height = canvas.height - canvas.height % CELL_SIZE;
    canvas.width = document.querySelector("body").getBoundingClientRect().width / 2;
    universe.height = Math.floor(canvas.height / CELL_SIZE);
    universe.width = Math.floor(canvas.width / CELL_SIZE);

    const ctx = canvas.getContext('2d');

    document.addEventListener("keydown", event => {
        console.log(event.shiftKey);
        if (universe.keypress(event.key, event.shiftKey || false)) {
            render();
        }
    });

    document.querySelector("#stop-selected").addEventListener("input", () => {
        universe.stop_selected(document.querySelector("#stop-selected").checked);
    });

    canvas.addEventListener("click", event => {
        const boundingRect = canvas.getBoundingClientRect();
        const scaleX = canvas.width / boundingRect.width;
        const scaleY = canvas.height / boundingRect.height;

        const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
        const canvasTop = (event.clientY - boundingRect.top) * scaleY;

        const row = Math.min(Math.floor(canvasTop / CELL_SIZE), canvas.height);
        const col = Math.min(Math.floor(canvasLeft / CELL_SIZE), canvas.width);

        if (universe.clicked(row, col, Math.round(canvasLeft), Math.round(canvasTop))) {
            render();
        }
    });

    const render = () => {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        universe.draw_grid();
        universe.draw_points();
        //window.raf_id = requestAnimationFrame(render);
    };

    window.js = js;
    window.memory = memory;
    window.render =render;
    window.raf_id = requestAnimationFrame(render);
});
});
