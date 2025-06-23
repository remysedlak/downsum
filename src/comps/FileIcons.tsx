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
            
            <RiFolder2Line  
                className="hover:text-green-500 hover:cursor-pointer w-5 h-5 rounded-full absolute right-24"
                onClick={() => showInExplorer(path)}
            />                
        </div>
    );
};

export default FileIcons;