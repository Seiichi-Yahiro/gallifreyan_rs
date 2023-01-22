const RON = {
    description: "Rusty Object Notation",
    accept: {
        "text/plain": [".ron"]
    }
};

const SVG = {
    description: "Scalable Vector Graphics",
    accept: {
        "image/svg+xml": [".svg"]
    }
}

export const openRONFile = async () => {
    const options = {
        multiple: false,
        excludeAcceptAllOption: true,
        types: [RON]
    };

    const [fileHandle] = await window.showOpenFilePicker(options);
    return fileHandle;
    //const fileData = await fileHandle.getFile();
    //return fileData.arrayBuffer();
};

export const saveRONFile = async () => {
    const options = {
        excludeAcceptAllOption: true,
        types: [RON]
    };

    return await window.showSaveFilePicker(options);
};

export const saveSVGFile = async () => {
    const options = {
        excludeAcceptAllOption: true,
        types: [SVG]
    };

    return await window.showSaveFilePicker(options);
};

export const saveToFile = async (fileHandle, data) => {
    const writableStream = await fileHandle.createWritable();

    const blob = new Blob([data]);

    await writableStream.write(blob);
    await writableStream.close();
}