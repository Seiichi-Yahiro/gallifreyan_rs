export const openFile = async () => {
    const options = {
        multiple: false,
        excludeAcceptAllOption: true,
        types: [
            {
                description: "Rust Object Notation",
                accept: {
                    "text/plain": [".ron"]
                }
            }
        ]
    };

    const [fileHandle] = await window.showOpenFilePicker(options);
    const fileData = await fileHandle.getFile();
    return fileData.arrayBuffer();
};