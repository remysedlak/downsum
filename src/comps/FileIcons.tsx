import { RiDeleteBin6Line } from "react-icons/ri";
import { RiFolder2Line } from "react-icons/ri";
import { MdOutlineFileOpen } from "react-icons/md";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";

const showInExplorer = async (filePath: string) => {
  try {
    await invoke("show_in_folder", { path: filePath });
  } catch (error) {
    console.error("Failed to show in explorer:", error);
  }
};

const FileIcons = ({ path }: { path: string }) => {
    return (
        <div>
            <RiDeleteBin6Line className="hover:text-red-500 hover:cursor-pointer w-5 h-5 rounded-full absolute right-8"></RiDeleteBin6Line>
            <MdOutlineFileOpen
                className="hover:text-blue-500 hover:cursor-pointer w-5 h-5 rounded-full absolute right-16"
                onClick={() => {
                    console.log("Trying to open:", path);
                    openPath(path)
                        .then(() => console.log("Opened:", path))
                        .catch((err) =>
                            console.error("Failed to open:", err)
                        );
                }}
            />
            <RiFolder2Line  
                className="hover:text-green-500 hover:cursor-pointer w-5 h-5 rounded-full absolute right-24"
                onClick={() => showInExplorer(path)}
            />                
        </div>
    );
};

export default FileIcons;