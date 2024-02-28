#!/home/ki11errabbit/.cargo/bin/caat_shell



widepapers = $HOME ++ "/Pictures/Wallpapers/Widepapers"
tallpapers = $HOME ++ "/Pictures/Wallpapers/Tallpapers"

function select_widepaper(label, display) {
    list = find $widepapers "-name" "*.png" "-or" "-name" "*.jpeg" "-or" "-name" "*.jpg" "-or" "-name" "*.mp4" "-or" "-name" "*.webm" | shuf
    file = head $list
    echo $label ++ ": " ++ $file >> $HOME ++ "/wallpaper.log"
    return $file
}


function select_tallpaper(label, display) {
    list = find $tallpapers "-name" "*.png" "-or" "-name" "*.jpeg" "-or" "-name" "*.jpg" "-or" "-name" "*.mp4" "-or" "-name" "*.webm" | shuf
    file = head $list
    echo $label ++ ": " ++ $file >> $HOME ++ "/wallpaper.log"
    return $file
}


file1 = select_widepaper "wide1" "DP-1"
file2 = select_widepaper "wide2" "DP-2"
file3 = select_tallpaper "tall" "HDMI-A-1"

function apply_wallpaper(display, file) {
    trace "Applying wallpaper to " ++ $display
    return if contains $file ".png" then
        background {swaybg "-o" $display "-i" $file "-m" "fill"}
    else if contains $file ".jpeg" then
        background {swaybg "-o" $display "-i" $file "-m" "fill"}
    else if contains $file ".jpg" then
        background {swaybg "-o" $display "-i" $file "-m" "fill"}
    else if contains $file ".webm" then
        mpvpaper "-f" "-o" "no-audio loop" $display $file
    else if contains $file ".mp4" then
        mpvpaper "-f" "-o" "no-audio loop" $display $file
    else
        echo "Unsupported file type: " ++ $file
}


pkill "swaybg"
pkill "mpvpaper"

dp1 = apply_wallpaper "DP-1" $file1
dp2 = apply_wallpaper "DP-2" $file2
hdmi = apply_wallpaper "HDMI-A-1" $file3

join $dp1
join $dp2
join $hdmi
