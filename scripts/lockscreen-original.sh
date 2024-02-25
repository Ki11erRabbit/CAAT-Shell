#!/bin/sh
export widepapers="$HOME/Pictures/Wallpapers/Widepapers/"
export tallpapers="$HOME/Pictures/Wallpapers/Tallpapers/"
export widepapers_list="$HOME/.config/swaylock/wallpapers.txt"
export tallpapers_list="$HOME/.config/swaylock/tallpapers.txt"

# $1 label
# $2 display
function select_widepaper () {
    find "$widepapers" -name "*.png" -or -name "*.jpeg" -or -name "*.jpg" | shuf > "$widepapers_list"
    file=$(head -n 1 "$widepapers_list")
    echo "$1: $file" >> $HOME/lockscreen.log
    echo "$2:$file"

}

# $1 label
# $2 display
function select_tallpaper () {
    find "$tallpapers" -name "*.png" -or -name "*.jpeg" -or -name "*.jpg" | shuf > "$tallpapers_list"
    file=$(head -n 1 "$tallpapers_list")
    echo "$1: $file" >> $HOME/lockscreen.log
    echo "$2:$file"

}



sed '1d' -i "$widepapers_list"
sed '1d' -i "$tallpapers_list"

#if [[ -z $(grep '[^[:space:]]' $list) ]]; then
#    rm $widepapers_list
#    rm $tallpapers_list
#fi

rm $widepapers_list
rm $tallpapers_list

file1=$(select_widepaper "wide1" "DP-1")
file2=$(select_widepaper "wide2" "DP-2")
file3=$(select_tallpaper "tall" "HDMI-A-1")

swaylock  -ef -i "$file1" -i "$file2" -i "$file3"
