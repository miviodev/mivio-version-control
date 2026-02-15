
use std::{env, fs, io::{self, Read, Write}, path::{Path}};
use serde_json::{Value};
use tar::{Builder, Archive};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

  //            //
 // CONSTANTES //
//            //
// Paths
const SNAP_METADATA_PATH: &str = ".mvc/metadata";
const SNAP_ARCHIVE_PATH: &str = ".mvc/archives";
const IGNORE_FILE_PATH: &str = ".mvcignore";
const HEAD_PATH: &str = ".mvc/HEAD";
const USER_INFO_DIR: &str = ".muc"; // mivio user configs
const USER_INFO_FILE: &str = "user.json";
// Other
const STANDARD_IGNORE: &str = 
".mvc
.mvcignore";
const USER_INFOS: [&str;2] = ["email", "name"];
const BIN_NAME: &str = env!("CARGO_PKG_NAME");
  //            //
 // STRUCTURES //
//            //
#[derive(Serialize, Deserialize)]
struct Snapshot {
    hash: String,
    message: String,
    email: String,
    name: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct User {
    email: String,
    name: String,
}
  //                 //
 // IMPLEMENTATIONS //
//                 //
impl User {
    // fn new(name: &str, email: &str) -> User {
    //     User {email: email.to_string(), name: name.to_string()}
    // }
    fn new_from_name(name:&str) -> User {
        User { email: "None".to_string(), name: name.to_string()}
    }
    fn new_from_email(email:&str) -> User {
        User { email: email.to_string(), name: "None".to_string() }
    }
}
  //           //
 // FUNCTIONS //
//           //
/// вычисляет хеш файла
fn calculate_hash(path: &str) -> Result<String, std::io::Error>  {
    let mut file = fs::File::open(path)?; // открываем файл
    let mut hasher = Sha256::new(); // создаем хешер
    io::copy(&mut file, &mut hasher)?; // копируем контент из file в hasher
    Ok(format!("{:x}", hasher.finalize())) // возвращаеем форматируя как строку и заканчивая хеш
}
/// проверяет, есть ли инфа о пользователе (см константу USER_INFOS для подробности о инфе)
fn is_user_info() -> bool {
    if let Some(home) = std::env::home_dir() { // если Some(home) будет равен home dir
        let full_path = home.join(USER_INFO_DIR).join(USER_INFO_FILE); // то сделай полный путь до файла
        if full_path.exists() { // если он существует
            return true; 
        } else { 
            return false;
        }
    }
    false
}
/// Меняем данные пользователя.
fn config_user(target:&str, data: &str) -> Result<(), io::Error> {
    if is_user_info() { // если существуют данные пользователя
        if let Some(home) = std::env::home_dir() { 
            let path = home.join(USER_INFO_DIR).join(USER_INFO_FILE);
            let file = fs::read_to_string(&path)?; // читаем пользователя
            let mut value: Value = serde_json::from_str(&file)?; // парсим json
            if USER_INFOS.contains(&target) { // если target есть в USER_INFOS
                value[target] = Value::String(data.to_string()); //то измени пункт json под названием [target] на data
                fs::write(path, serde_json::to_string(&value)?)?; // и запиши его обратно
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidInput,format!("invalid parameter {target}, available: {:?}", USER_INFOS))) // если например ввели emial а не email то верни ошибку
            }
        }
    } else { // если нет данных о юзере
        if let Some(home) = std::env::home_dir() {
            let path = home.join(USER_INFO_DIR);
            fs::create_dir_all(&path)?; //создаем путь до файла
            let mut info = fs::File::create(&path.join(USER_INFO_FILE))?; // создай файл
            if target == "name" { // если target будет равен имени
                let user = User::new_from_name(data); // то создай нового пользователя из имени
                let json_format: String = serde_json::to_string(&user)?; // создай json в строку
                info.write(json_format.as_bytes())?; // запиши в него наш json
            } else if target == "email" {
                let user = User::new_from_email(data);
                let json_format: String = serde_json::to_string(&user)?;
                info.write(json_format.as_bytes())?;
            }
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to open home directory"))
        }
    }
    Ok(())
}

/// получение данных пользователя
fn get_user() -> Result<User, io::Error>{
    let path = env::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Unable to find home directory"))?.join(USER_INFO_DIR).join(USER_INFO_FILE);
    if is_user_info() {
        println!("{}", path.display());
        let file = fs::read_to_string(path)?;
        let value: Value = serde_json::from_str(&file)?;
        
        let email = value["email"].as_str().ok_or_else(||
            io::Error::new(
                io::ErrorKind::Other,
                format!("[{}] Failed to convert serde_json::Value::String to &str",
                "ERROR".red())))?.to_string();
        let name = value["name"].as_str().ok_or_else(||
            io::Error::new(
                io::ErrorKind::Other,
                format!("[{}] Failed to convert serde_json::Value::String to &str",
                "ERROR".red())))?.to_string();
        let usr = User {
            email: email,
            name: name
        };
        println!("{:?}", &usr);
        Ok(usr)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, format!("No user information found. Search path: {}\ntry execute:
    {} cfg email your@email
    {} cfg name your_name", path.display(), BIN_NAME.bright_green(), BIN_NAME.bright_green())))
    }
}
/// проверяет, надо ли игнорировать путь
fn should_ignore(path: &Path, ignore_list: &[&str]) -> bool{
    if !path.is_absolute() { // если путь не абсолютный
        let ancestors = path.ancestors(); // Создает итератор по объекту Path и его предкам.
        
        for ancestor in ancestors { // берем все элементы
            if &ancestor == &Path::new(".") {return true;} // если путь это . (текущая директория) то игнорируй
            // let ancestor = ancestor.strip_prefix("./").unwrap();
            if ignore_list.iter().any(|ignore| {
                //println!("{} == {}: {}", *ignore, ancestor.as_os_str().to_str().unwrap(), Some(*ignore) == ancestor.as_os_str().to_str());
                let ignore_path = Some(*ignore) == ancestor.as_os_str().to_str();// если в игнор листе будет наш путь
                return ignore_path;  
            }){
                return true; // то возвращаем true (да, игнорировать)
            }
        }
    } else {
        return true; // если путь будет абсолютным то есть риск удалить системные файлы, поэтому лучше будет игнорить
    }
    return false; // ну а если вапще чета как та и не то и не другое то false
}
/// Проверяет в репозитории ли мы
fn is_in_repo() -> Result<bool, io::Error> {
    let folder = fs::exists(".mvc");
    folder
}
/// Получение ignorelist'а
fn get_ignore() -> Result<String, io::Error> {
    if !is_in_repo()? {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Not in repo"));
    }
    let ignore = fs::read_to_string(IGNORE_FILE_PATH);
    ignore
}
/// удаляет все то что не в игнор листе
fn delete_current(path: &Path) -> Result<(), std::io::Error>  {
    let current_dir = walkdir::WalkDir::new(path).min_depth(1).contents_first(true); // читаем текущую директорию
    let ignore = get_ignore(); // получаем ignorelist
    let ignore: String = ignore?;
    let splited_ignore: Vec<&str> = ignore.split("\n").collect(); //разделяем на строки
    
    for entry in current_dir {
        
        let borrowed_entry = entry?; 
        let should_ignore = !should_ignore(borrowed_entry.path().strip_prefix("./").unwrap_or_else(|e|{
            eprintln!("[{}] {e}", "ERROR".red());
            Path::new(borrowed_entry.path())
        }), &splited_ignore);
        //println!("DELETECURRENT y: {}: {}", y.path().strip_prefix("./").unwrap().display(), z);
        if should_ignore {
            if borrowed_entry.metadata()?.is_dir() {
                delete_current(borrowed_entry.path())?;
            } else {
                fs::remove_file(borrowed_entry.path())?;
            }
        }
    }
    Ok(())
}
/// Функция создающая архив с кодом
fn create_archive(file: fs::File) -> Result<(), std::io::Error> {
    let ignore = get_ignore(); // получаем ignorelist
    let ignore = ignore?;
    let splited_ignore: Vec<&str> = ignore.split("\n").collect(); //разделяем на строки
    // let mut objects: Vec<String> = vec![];
    let mut archive = Builder::new(file); // Создаем архив в файле
    let current_dir = fs::read_dir("."); // читаем текущую строку
    for object in current_dir? {
        let object = object?; // берем объект
        let object_name = object.file_name(); // берем имя в виде строки
        if !splited_ignore.iter().any(|f| *f==object_name.as_os_str()) { // если строка ignore != файл то
            if object.metadata()?.is_dir() { //проверь директория ли это
                archive.append_dir_all(&object_name, &object_name)?; // и заархивируй ее с дочерними элементами
            }
            else { 
                archive.append_path(object_name)?; // просто заархивируй
            }
        }
    }
    Ok(())
}
/// функция создающая архив с json (снапшот)
fn create_snap(snap_id: u32, message: &str) -> Result<(), std::io::Error> {
    create_archive(fs::File::create(format!("{}/{}.tar", SNAP_ARCHIVE_PATH, snap_id))?)?;
    let hash = calculate_hash(&format!("{}/{}.tar", SNAP_ARCHIVE_PATH, snap_id))?;
    let user = get_user()?;
    let snapshot = Snapshot{
        hash: hash.to_string(),
        message: message.to_string(),
        email: user.email,
        name: user.name
    };
    let json_format: String = serde_json::to_string(&snapshot)?;
    let mut info = fs::File::create(format!("{}/{}.json", SNAP_METADATA_PATH, snap_id))?;
    info.write(json_format.as_bytes())?;
    Ok(())
}
/// Иницилизация нового репозитория
fn init() -> Result<(), std::io::Error> {
    if is_in_repo()? {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, "The repository has already been initialized."));
    } else {
        fs::create_dir_all(SNAP_ARCHIVE_PATH)?;  // } создаем папки
        fs::create_dir_all(SNAP_METADATA_PATH)?; // }
        let mut ignore = fs::File::create(IGNORE_FILE_PATH)?; // создаем ignore list
        ignore.write(STANDARD_IGNORE.as_bytes())?; // записываем его
        // create_snap(1 , "Initial")?; // создаем снапшот
        let mut head = fs::File::create(HEAD_PATH)?;
        head.write("0".as_bytes())?;
        println!("Repository initialized! Please execute \"{} save Initial\" for create first commit!", BIN_NAME.bright_green());
        if !is_user_info() {
            eprintln!("[{}] set information about you
    {} cfg name your_name
    {} cfg email your@email", "WARNING".yellow(), BIN_NAME.bright_green(), BIN_NAME.bright_green())
        }
    }
    Ok(())
}
/// распаковка архива с данными по id
fn unpack_arch(id: &u32) -> Result<(), std::io::Error> {
    delete_current(&Path::new("."))?;
    let path = format!("{}/{}.tar", SNAP_ARCHIVE_PATH, id);
    let file = fs::File::open(path)?;
    let mut archive = Archive::new(file);
    archive.unpack(".")?;
    Ok(())
}
/// Функция для возврата к снапшоту
fn return_to_snap(id: u32) -> Result<(), std::io::Error> {
    if !is_in_repo()? {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Not in repo"));
    }
    let new_hash = calculate_hash(&format!("{}/{}.tar", SNAP_ARCHIVE_PATH, id))?;
    // получаем его метаданные
    let mut metadata: String = String::new();
    let mut metadata_file = fs::File::open(format!("{}/{}.json", SNAP_METADATA_PATH, id))?;
    metadata_file.read_to_string(&mut metadata)?;
    let metadata: Value = serde_json::from_str(&metadata)?;
    if metadata["hash"].as_str().ok_or_else(||io::Error::new(io::ErrorKind::Other, 
        format!("[{}] Failed to convert serde_json::Value::String to &str", "ERROR".red())))?.to_string() != new_hash {
        return Err(std::io::Error::new(io::ErrorKind::Other, "Hashs not match"))
    }
    // распаковываем архив...
    unpack_arch(&id)?;
    // выводим сообщение:
    println!("Message: {}", metadata["message"]);
    Ok(())
}
/// Парсим последний коммит
fn parse_last_snap_id() -> Result<u32, io::Error> {
    let head_content = fs::read_to_string(HEAD_PATH)?; // читаем head
    let head_massive: Vec<&str> = head_content.split("\n").collect(); // разделяем на строки
    let last_snap_str = head_massive[0]; // берем первую строку (номер послед. коммита)
    let last_snap_int: u32 = last_snap_str.parse().ok().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Unable to parse"))?;
    return Ok(last_snap_int);
}
/// Функция для того чтобы цифарки обновить снапшота и тд
fn save_snap(message: &str) -> Result<(), std::io::Error> {
    if !is_in_repo()? {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Not in repo"));
    }
    let last_snap = parse_last_snap_id()?;
    let last_snap = last_snap + 1;
    
    create_snap(last_snap, message)?;
    fs::write(HEAD_PATH, last_snap.to_string())?;
    println!("Saved!");
    Ok(())
}
/// Вывод всех снапшотов с их инфой
fn read_all_snaps() -> Result<(), std::io::Error> {
    if !is_in_repo()? {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Not in repo"));
    }
    let mut paths: Vec<_> = fs::read_dir(SNAP_METADATA_PATH)?.filter_map(|entry| entry.ok())
        .collect();
    paths.sort_by_key(|key| {
        key.file_name()
    });
    for path in paths {
        let file_name = path.file_name();
        let file = fs::read_to_string(path.path())?;
        let value: Value = serde_json::from_str(&file)?;
        let pretty_name = file_name.to_str().unwrap_or("ERROR").replace(".json", "");
        println!("{}: {}
{}:        {}
Message:     {}
{}:       {}
{}:    {}
----", "Snapshot ID".bright_cyan(), pretty_name,"Hash".bright_purple(), value["hash"],value["message"],  "Email".green(), value["email"], "Username".yellow(), value["name"]);
        }
    Ok(())
}
/// основная функция.
fn run() -> Result<(), std::io::Error>  {
    let args: Vec<String> = std::env::args().collect();
    let version = || {println!("{} v{}. Licensed under {} license", BIN_NAME.bright_green(), env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_LICENSE"))};
    let usage = || {println!(
"Usage:
    mvc [-v | --version] <command> [<args>]
Commands:
    cfg <target> <data> - change data about you
    init                - initialize a new repository
    log                 - display all snapshots
    return <id>         - returns to <id> version
    save <message>      - saves version
    help                - show this message")};
    if args.len() == 2 {
        if args[1] == "init" {
            init()?;
        } else if args[1] == "log" {
            read_all_snaps()?;
        } else if args[1] == "--version" || args[1] == "-v"{
            version()
        } else{usage()}
    } else if args.len() >= 3 {
        if args[1] == "return" {
            let id:Option<u32>= args[2].parse().ok();
            let id = id.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to parse id"))?;
            return_to_snap(id)?;
        } else if args[1] == "save" {
            save_snap(&args[2..].join(" "))?;   
        }else if args.len() == 4 || args[1] == "cfg"{
            config_user(&args[2], &args[3])?;
        } else {usage()} 
    } else {usage()}
    Ok(())
}
fn main() {
    if let Err(e) = run() { // если Err(e) [(e это io::Error)] будет равен вызванной функции run()
        eprintln!("[{}] {}","ERROR".red(), e); // то выведи e (ошибку)
        std::process::exit(1); // и заверши выполнение с кодом 1
    }
}