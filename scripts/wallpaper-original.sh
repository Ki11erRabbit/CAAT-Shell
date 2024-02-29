#!/bin/sh

case $1 in
    start) while true; do
            /home/ki11errabbit/.local/bin/wallpaper-wayland.sh
            sleep 300
        done
        ;;
    *);;
esac


widepapers="$HOME/Pictures/Wallpapers/Widepapers/"
tallpapers="$HOME/Pictures/Wallpapers/Tallpapers/"
widepapers_list="$HOME/.config/swaylock/wallpapers.txt"
tallpapers_list="$HOME/.config/swaylock/tallpapers.txt"

# $1 label
# $2 display
function select_widepaper () {
    file=$(find "$widepapers" -name "*.png" -or -name "*.jpeg" -or -name "*.jpg" -or -name "*.webm" -or -name "*.mp4" | shuf | head -n 1)
    echo "$1: $file" >> $HOME/wallpaper.log
    echo "$file"

}

# $1 label
# $2 display
function select_tallpaper () {
    file=$(find "$tallpapers" -name "*.png" -or -name "*.jpeg" -or -name "*.jpg" -or -name "*.webm" -or -name "*.mp4" | shuf | head -n 1)
    echo "$1: $file" >> $HOME/wallpaper.log
    echo "$file"

}

file1=$(select_widepaper "wide1")
file2=$(select_widepaper "wide2")
file3=$(select_tallpaper "tall")


# $1 display
# $2 file
function apply_wallpaper () {
    #mpvpaper -f -o "no-audio loop" $1 "$2"
    case $2 in
        *.png|*.jpeg|*.jpg)
            swaybg -o $1 -i "$2" -m fill  &
            ;;
        *.webm|*.mp4)
            mpvpaper -f -o "no-audio loop" $1 "$2"
            ;;
    esac
}

pkill -x swaybg
pkill -x mpvpaper
apply_wallpaper "DP-1" "$file1"
apply_wallpaper "DP-2" "$file2"
apply_wallpaper "HDMI-A-1" "$file3"
#swaylock  -ef -i "$file1" -i "$file2" -i "$file3"
