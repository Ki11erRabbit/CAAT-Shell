#!/home/ki11errabbit/.cargo/bin/caat_shell

widepapers = $HOME ++ "/Pictures/Wallpapers/Widepapers"
tallpapers = $HOME ++ "/Pictures/Wallpapers/Tallpapers"

function select_widepaper(label, display) {
    list = find $widepapers "-name" "*.png" "-or" "-name" "*.jpeg" "-or" "-name" "*.jpg" | shuf
    file = head $list
    echo $label ++ ": " ++ $file >> $HOME ++ "/lockscreen.log"
    return $display ++ ":" ++ $file
}


function select_tallpaper(label, display) {
    list = find $tallpapers "-name" "*.png" "-or" "-name" "*.jpeg" "-or" "-name" "*.jpg" | shuf
    file = head $list
    echo $label ++ ": " ++ $file >> $HOME ++ "/lockscreen.log"
    return $display ++ ":" ++ $file
}



file1 = select_widepaper "wide1" "DP-1"
file2 = select_widepaper "wide2" "DP-2"
file3 = select_tallpaper "tall" "HDMI-A-1"
swaylock "-ef" "-i" $file1 "-i" $file2 "-i" $file3

