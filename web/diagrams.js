// PlantUML / Graphviz rendering for omd Web preview and export.

const OMD_PLANTUML_ALPHABET =
    '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_';

function omdPlantumlBase64(data) {
    let out = '';
    for (let i = 0; i < data.length; i += 3) {
        const b1 = data[i];
        const b2 = i + 1 < data.length ? data[i + 1] : 0;
        const b3 = i + 2 < data.length ? data[i + 2] : 0;
        out += OMD_PLANTUML_ALPHABET[b1 >> 2];
        out += OMD_PLANTUML_ALPHABET[((b1 & 0x3) << 4) | (b2 >> 4)];
        out += OMD_PLANTUML_ALPHABET[((b2 & 0xf) << 2) | (b3 >> 6)];
        out += OMD_PLANTUML_ALPHABET[b3 & 0x3f];
    }
    return out;
}

function omdEncodePlantuml(text) {
    if (typeof pako === 'undefined') return null;
    const deflated = pako.deflateRaw(text, { level: 9 });
    return omdPlantumlBase64(deflated);
}

function omdGetDiagramSource(node, attr) {
    const saved = node.getAttribute(attr);
    if (saved) return saved;
    if (node.querySelector('svg, img')) return '';
    const source = (node.textContent || '').trim();
    if (source) node.setAttribute(attr, source);
    return source;
}

let omdPlantumlGen = 0;
window.omdRenderPlantuml = async function () {
    const gen = ++omdPlantumlGen;
    const nodes = document.querySelectorAll('.preview-content .plantuml, article .plantuml');
    for (const node of nodes) {
        if (gen !== omdPlantumlGen) return;
        const source = omdGetDiagramSource(node, 'data-plantuml-source');
        if (!source) continue;
        if (node.getAttribute('data-processed') === '1' && node.querySelector('img, svg')) continue;

        const encoded = omdEncodePlantuml(source);
        if (!encoded) {
            node.textContent = source;
            continue;
        }
        const url = 'https://www.plantuml.com/plantuml/svg/~1' + encoded;
        node.innerHTML = '';
        const img = document.createElement('img');
        img.alt = 'PlantUML diagram';
        img.loading = 'lazy';
        img.src = url;
        img.onerror = () => {
            node.textContent = source;
            node.removeAttribute('data-processed');
        };
        img.onload = () => node.setAttribute('data-processed', '1');
        node.appendChild(img);
    }
};

let omdGraphvizGen = 0;
window.omdRenderGraphviz = async function () {
    if (typeof Viz === 'undefined') return;
    const gen = ++omdGraphvizGen;
    const nodes = document.querySelectorAll('.preview-content .graphviz, article .graphviz');
    for (const node of nodes) {
        if (gen !== omdGraphvizGen) return;
        const source = omdGetDiagramSource(node, 'data-graphviz-source');
        if (!source) continue;
        if (node.getAttribute('data-processed') === '1' && node.querySelector('svg')) continue;

        node.innerHTML = '';
        node.textContent = source;
        try {
            const viz = new Viz();
            const svg = await viz.renderSVGElement(source);
            if (gen !== omdGraphvizGen) return;
            node.innerHTML = '';
            node.appendChild(svg);
            node.setAttribute('data-processed', '1');
        } catch (e) {
            console.warn('graphviz render:', e);
            node.textContent = source;
            node.removeAttribute('data-processed');
        }
    }
};

window.omdRenderDiagrams = async function () {
    await window.omdRenderPlantuml();
    await window.omdRenderGraphviz();
};
