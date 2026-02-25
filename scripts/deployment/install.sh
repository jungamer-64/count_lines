#!/usr/bin/env bash
# install_count_lines.sh (v2.2.0 - refactored)
# count_lines: ファイル行数/文字数/単語数の集計ツール

set -euo pipefail

# ========================================
# グローバル定数
# ========================================
readonly SCRIPT_VERSION="2.2.0"
# shellcheck disable=SC2034
# SCRIPT_VERSION is kept for informational/packaging purposes

# ========================================
# インストーラー設定
# ========================================
INSTALL_MODE="user"   # "user" or "global"
UNINSTALL="no"
DRYRUN="no"

# ========================================
# ユーティリティ関数
# ========================================
err() { echo "Error: $*" >&2; exit 1; }
info() { echo "$*" >&2; }

check_command() {
  command -v "$1" >/dev/null 2>&1 || err "'$1' not found."
}

check_required_commands() {
  local cmds=(find xargs wc awk sort grep)
  for cmd in "${cmds[@]}"; do
    check_command "$cmd"
  done
}

validate_options() {
  if [[ "$DRYRUN" == yes && "$UNINSTALL" == yes ]]; then
    err "--dry-run と --uninstall は同時に使えません"
  fi
}

# ========================================
# PATH設定関数
# ========================================
ensure_path_in_rc() {
  local rc_file="$1"
  # shellcheck disable=SC2016
  # keep the literal $HOME here so the target rc file receives an expansion at runtime
  local path_line='export PATH="$HOME/.local/bin:$PATH"'
  
  [[ -f "$rc_file" ]] || touch "$rc_file"
  
  if ! grep -Fq "$path_line" "$rc_file"; then
    printf '\n# add user local bin to PATH for count_lines v2\n%s\n' "$path_line" >> "$rc_file"
    info "Updated: $rc_file (added ~/.local/bin to PATH)"
  fi
}

setup_shell_path() {
  case "${SHELL##*/}" in
    bash)
      if [[ "${OSTYPE:-}" == darwin* ]]; then
        ensure_path_in_rc "$HOME/.bash_profile"
      else
        ensure_path_in_rc "$HOME/.bashrc"
      fi
      ;;
    zsh)
      ensure_path_in_rc "$HOME/.zprofile"
      ensure_path_in_rc "$HOME/.zshrc"
      ;;
    *)
      ensure_path_in_rc "$HOME/.profile"
      ;;
  esac
}

add_to_current_path() {
  if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    export PATH="$HOME/.local/bin:$PATH"
    info "Temporarily added ~/.local/bin to current PATH (for this session)."
  fi
}

# ========================================
# count_lines 本体生成
# ========================================
emit_count_lines() {
  cat <<'EOF'
#!/usr/bin/env bash
# count_lines v2.2.0 — ファイル行数/文字数(＋任意で単語数)の集計ツール

set -euo pipefail
LC_ALL=C

readonly VERSION="2.2.0"

# ========================================
# デフォルト設定
# ========================================
FORMAT="table"
SORT_KEY="lines"
SORT_DESC=0
TOP_N=0
BY_MODE=""
DIR_DEPTH=1
SUMMARY_ONLY=0
TOTAL_ONLY=0
HIDDEN=0
FOLLOW=0
USE_GIT=0
MAX_SIZE=""
JOBS=""
INCLUDE_PATTERNS=()
EXCLUDE_PATTERNS=()
INCLUDE_PATHS=()
EXCLUDE_PATHS=()
EXCLUDE_DIRS=()
EXT_FILTERS=()
PATHS=()
NO_COLOR=0
TEXT_ONLY=0
FILES_FROM=""
FILES_FROM0=""
MIN_LINES=""
MAX_LINES=""
MIN_CHARS=""
MAX_CHARS=""
WORDS=0
MIN_WORDS=""
MAX_WORDS=""
DEFAULT_PRUNE=1
ABS_PATH=0
TRIM_ROOT=""
MTIME_SINCE=""
MTIME_UNTIL=""

# ========================================
# ユーティリティ関数
# ========================================
err() { echo "Error: $*" >&2; }
info() { echo "$*" >&2; }
is_tty() { [[ -t 1 ]]; }

color() {
  local name="$1"; shift || true
  if [[ $NO_COLOR -eq 1 || $FORMAT != table || ! -t 1 ]]; then
    printf "%s" "$*"
    return
  fi
  case "$name" in
    dim) printf "\033[2m%s\033[0m" "$*" ;;
    bold) printf "\033[1m%s\033[0m" "$*" ;;
    green) printf "\033[32m%s\033[0m" "$*" ;;
    yellow) printf "\033[33m%s\033[0m" "$*" ;;
    cyan) printf "\033[36m%s\033[0m" "$*" ;;
    *) printf "%s" "$*" ;;
  esac
}

cpu_cores() {
  if command -v nproc >/dev/null 2>&1; then
    nproc
  elif [[ "${OSTYPE:-}" == darwin* ]] && command -v sysctl >/dev/null 2>&1; then
    sysctl -n hw.ncpu
  else
    echo 4
  fi
}

# ========================================
# サイズ変換関数
# ========================================
size_to_find_arg() {
  local s="$1" sign="" num unit mul=1
  [[ -z "$s" ]] && { echo ""; return; }
  
  if [[ "$s" =~ ^([+-]?)([0-9]+)(b|c|w|k|M|G|T|P)$ ]]; then
    sign="${BASH_REMATCH[1]}"
    num="${BASH_REMATCH[2]}"
    unit="${BASH_REMATCH[3]}"
    [[ -z "$sign" ]] && sign="-"
    echo "${sign}${num}${unit}"
    return
  fi
  
  case "$s" in
    +*) sign="+"; s="${s#+}";;
    -*) sign="-"; s="${s#-}";;
    *)  sign="-";;
  esac
  
  if [[ "$s" =~ ^([0-9]+)([KMGTP]i?B?|[KMGTPkmgtp])?$ ]]; then
    num="${BASH_REMATCH[1]}"
    unit="${BASH_REMATCH[2]}"
    case "$unit" in
      K|KiB|KB|k|kB|kib) mul=1024;;
      M|MiB|MB|m|mB|mib) mul=$((1024*1024));;
      G|GiB|GB|g|gB|gib) mul=$((1024*1024*1024));;
      T|TiB|TB|t|tB|tib) mul=$((1024*1024*1024*1024));;
      P|PiB|PB|p|pB|pib) mul=$((1024*1024*1024*1024*1024));;
      "") mul=1;;
      *) echo "$1"; return;;
    esac
    echo "${sign}$((num*mul))c"
  else
    echo "$1"
  fi
}

stat_size() {
  local sz
  if sz=$(stat -c %s "$1" 2>/dev/null); then
    printf '%s\n' "$sz"
    return 0
  fi
  if sz=$(stat -f %z "$1" 2>/dev/null); then
    printf '%s\n' "$sz"
    return 0
  fi
  if sz=$(wc -c <"$1" 2>/dev/null); then
    printf '%s\n' "$sz"
    return 0
  fi
  return 1
}

supports_newermt() {
  if [[ -n "${__HAS_NEWERMT:-}" ]]; then
    return "$__HAS_NEWERMT"
  fi
  if find . -maxdepth 0 -newermt "1970-01-01" -print -quit >/dev/null 2>&1; then
    __HAS_NEWERMT=0
    return 0
  fi
  __HAS_NEWERMT=1
  return 1
}

# ========================================
# フィルタリング関数
# ========================================
should_exclude_hidden() {
  local f="$1"
  [[ $HIDDEN -eq 0 ]] || return 1
  case "$f" in
    .*|*/.*) return 0 ;;
    *) return 1 ;;
  esac
}

should_exclude_dir() {
  local f="$1"
  [[ ${#EXCLUDE_DIRS[@]} -eq 0 ]] && return 1
  
  local d
  for d in "${EXCLUDE_DIRS[@]}"; do
    [[ "$f" == $d || "$f" == $d/* ]] && return 0
  done
  return 1
}

matches_include_paths() {
  local f="$1"
  [[ ${#INCLUDE_PATHS[@]} -eq 0 ]] && return 0
  
  local p
  for p in "${INCLUDE_PATHS[@]}"; do
    [[ "$f" == $p ]] && return 0
  done
  return 1
}

matches_exclude_paths() {
  local f="$1"
  [[ ${#EXCLUDE_PATHS[@]} -eq 0 ]] && return 1
  
  local p
  for p in "${EXCLUDE_PATHS[@]}"; do
    [[ "$f" == $p ]] && return 0
  done
  return 1
}

matches_include_patterns() {
  local f="$1"
  [[ ${#INCLUDE_PATTERNS[@]} -eq 0 ]] && return 0
  
  local base="${f##*/}"
  local p
  for p in "${INCLUDE_PATTERNS[@]}"; do
    [[ "$base" == $p ]] && return 0
  done
  return 1
}

matches_exclude_patterns() {
  local f="$1"
  [[ ${#EXCLUDE_PATTERNS[@]} -eq 0 ]] && return 1
  
  local base="${f##*/}"
  local p
  for p in "${EXCLUDE_PATTERNS[@]}"; do
    [[ "$base" == $p ]] && return 0
  done
  return 1
}

matches_ext_filters() {
  local f="$1"
  [[ ${#EXT_FILTERS[@]} -eq 0 ]] && return 0
  
  local lower="${f,,}"
  local ex
  for ex in "${EXT_FILTERS[@]}"; do
    ex="${ex#.}"
    ex="${ex,,}"
    [[ "$lower" == *."$ex" ]] && return 0
  done
  return 1
}

matches_size_filter() {
  local f="$1"
  [[ -z "$MAX_SIZE" ]] && return 0
  
  local MAX_ARG SIGN="" LIMIT="" sz
  MAX_ARG="$(size_to_find_arg "$MAX_SIZE")"
  
  if [[ "$MAX_ARG" =~ ^([+-]?)([0-9]+)c$ ]]; then
    SIGN="${BASH_REMATCH[1]}"
    LIMIT="${BASH_REMATCH[2]}"
  else
    return 0
  fi
  
  sz=$(stat_size "$f" 2>/dev/null || true)
  [[ -z "${sz:-}" ]] && return 1
  
  case "$SIGN" in
    +) (( sz <= LIMIT )) && return 1 ;;
    -|"") (( sz > LIMIT )) && return 1 ;;
  esac
  return 0
}

filter_path_list() {
  while IFS= read -r -d '' f; do
    should_exclude_hidden "$f" && continue
    should_exclude_dir "$f" && continue
    matches_include_paths "$f" || continue
    matches_exclude_paths "$f" && continue
    matches_include_patterns "$f" || continue
    matches_exclude_patterns "$f" && continue
    matches_ext_filters "$f" || continue
    matches_size_filter "$f" || continue
    
    printf '%s\0' "$f"
  done
}

# ========================================
# ファイルリスト生成関数
# ========================================
list_from_file() {
  if [[ -n "$FILES_FROM0" ]]; then
    cat -- "$FILES_FROM0" | filter_path_list
    return
  fi
  
  if [[ -n "$FILES_FROM" ]]; then
    while IFS= read -r line || [[ -n "${line}" ]]; do
      [[ -n "$line" ]] && printf '%s\0' "$line"
    done < "$FILES_FROM" | filter_path_list
    return
  fi
  
  return 1
}

list_from_git() {
  [[ $USE_GIT -eq 0 ]] && return 1
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || return 1
  
  git ls-files -z --cached --others --exclude-standard -- "${PATHS[@]}" | filter_path_list
  return 0
}

build_find_prune_args() {
  local -n prune_ref=$1
  
  if [[ $DEFAULT_PRUNE -eq 1 ]]; then
    prune_ref=(
      -path '*/.git' -o -path '*/.hg' -o -path '*/.svn' -o
      -path '*/node_modules' -o -path '*/.venv' -o -path '*/venv' -o
      -path '*/build' -o -path '*/dist' -o -path '*/target' -o
      -path '*/.cache' -o -path '*/.direnv' -o -path '*/.mypy_cache' -o
      -path '*/.pytest_cache' -o -path '*/coverage' -o -path '*/__pycache__' -o
      -path '*/.idea' -o -path '*/.next' -o -path '*/.nuxt'
    )
  else
    prune_ref=(-false)
  fi
  
  [[ $HIDDEN -eq 0 ]] && prune_ref+=(-o -path '*/.*')
  
  if [[ ${#EXCLUDE_DIRS[@]} -gt 0 ]]; then
    prune_ref+=(-o \()
    local d
    for d in "${EXCLUDE_DIRS[@]}"; do
      prune_ref+=(-path "$d" -o)
    done
    prune_ref+=(-path "" \))
  fi
}

build_find_file_conditions() {
  local -n cond_ref=$1
  
  [[ $HIDDEN -eq 0 ]] && cond_ref+=(-not -name '.*')
  
  if [[ ${#INCLUDE_PATTERNS[@]} -gt 0 ]]; then
    cond_ref+=(\()
    local p
    for p in "${INCLUDE_PATTERNS[@]}"; do
      cond_ref+=(-name "$p" -o)
    done
    cond_ref+=(-name "" \))
  fi
  
  local p
  for p in "${EXCLUDE_PATTERNS[@]:-}"; do
    cond_ref+=(-not -name "$p")
  done
  
  if [[ ${#INCLUDE_PATHS[@]} -gt 0 ]]; then
    cond_ref+=(\()
    local ip
    for ip in "${INCLUDE_PATHS[@]}"; do
      cond_ref+=(-path "$ip" -o)
    done
    cond_ref+=(-path "" \))
  fi
  
  for p in "${EXCLUDE_PATHS[@]:-}"; do
    cond_ref+=(-not -path "$p")
  done
  
  if [[ ${#EXT_FILTERS[@]} -gt 0 ]]; then
    cond_ref+=(\()
    local ex
    for ex in "${EXT_FILTERS[@]}"; do
      ex="${ex#.}"
      cond_ref+=(-iname "*.${ex}" -o)
    done
    cond_ref+=(-name "" \))
  fi
  
  if [[ -n "$MAX_SIZE" ]]; then
    local SZ
    SZ="$(size_to_find_arg "$MAX_SIZE")"
    cond_ref+=(-size "$SZ")
  fi
  
  if supports_newermt; then
    [[ -n "$MTIME_SINCE" ]] && cond_ref+=(-newermt "$MTIME_SINCE")
    [[ -n "$MTIME_UNTIL" ]] && cond_ref+=(! -newermt "$MTIME_UNTIL")
  fi
}

list_from_find() {
  local FFLAGS=()
  [[ $FOLLOW -eq 1 ]] && FFLAGS+=(-L)
  
  local PRUNE=()
  build_find_prune_args PRUNE
  
  local FILECOND=()
  build_find_file_conditions FILECOND
  
  find "${PATHS[@]}" "${FFLAGS[@]}" \( "${PRUNE[@]}" \) -prune -o \( -type f "${FILECOND[@]}" -print0 \)
}

list_files() {
  list_from_file && return
  list_from_git && return
  list_from_find
}

# ========================================
# 計測関数
# ========================================
measure_wc() {
  PWD_ABS="$PWD" WORDS="$WORDS" ABS_PATH="$ABS_PATH" TRIM_ROOT="$TRIM_ROOT" TEXT_ONLY="$TEXT_ONLY" \
  xargs -0 -n 1 -P "$JOBS" sh -c '
    f=$1
    [ -r "$f" ] || exit 0
    
    if [ "${TEXT_ONLY:-0}" -eq 1 ]; then
      if command -v grep >/dev/null 2>&1; then
        LC_ALL=C grep -Iq . "$f" || exit 0
      fi
    fi
    
    outpath="$f"
    case "$ABS_PATH" in
      1) case "$outpath" in
           /*) ;;
           *) outpath="$PWD_ABS/$outpath";;
         esac
         ;;
    esac
    
    if [ -n "$TRIM_ROOT" ]; then
      case "$outpath" in
        "$TRIM_ROOT"/*) outpath="${outpath#"$TRIM_ROOT"/}";;
      esac
    fi
    
    if [ "$WORDS" -eq 1 ]; then
      out=$({ wc -l -w -m <"$f" 2>/dev/null || wc -l -w -c <"$f" 2>/dev/null; } || exit 0)
      set -- $out
      l=${1:-}; w=${2:-}; m=${3:-}
      [ -n "$l" ] && [ -n "$m" ] && printf "%s\t%s\t%s\t%s\n" "$l" "$m" "$w" "$outpath"
    else
      out=$({ wc -l -m <"$f" 2>/dev/null || wc -l -c <"$f" 2>/dev/null; } || exit 0)
      set -- $out
      l=${1:-}; m=${2:-}
      [ -n "$l" ] && [ -n "$m" ] && printf "%s\t%s\t%s\n" "$l" "$m" "$outpath"
    fi
  ' sh
}

# ========================================
# 後処理関数
# ========================================
postprocess() {
  local AWK_PROG
  AWK_PROG='
    BEGIN {
      by_mode="'"$BY_MODE"'"
      total_only='"$TOTAL_ONLY"'+0
      summary_only='"$SUMMARY_ONLY"'+0
      top='"$TOP_N"'+0
      sort_key="'"$SORT_KEY"'"
      sort_desc='"$SORT_DESC"'+0
      fmt="'"$FORMAT"'"
      minl="'"${MIN_LINES:-}"'"
      maxl="'"${MAX_LINES:-}"'"
      minc="'"${MIN_CHARS:-}"'"
      maxc="'"${MAX_CHARS:-}"'"
      words='"$WORDS"'+0
      minw="'"${MIN_WORDS:-}"'"
      maxw="'"${MAX_WORDS:-}"'"
      dir_depth='"$DIR_DEPTH"'+0
    }
    
    function tolower_s(s,   i,c,r){
      r=s
      for(i=1;i<=length(s);i++){
        c=substr(s,i,1)
        if(c>="A"&&c<="Z") r=substr(r,1,i-1)""tolower(c)""substr(r,i+1)
      }
      return r
    }
    
    function extname(path,   n,b,i,e) {
      n=split(path,a,"/")
      b=a[n]
      if (b ~ /^\.[^\.]+$/) {
        sub(/^\./, "", b)
        e=b
      } else {
        i=match(b,/\.[^\.]+$/)
        e=i?substr(b,i+1):""
      }
      return tolower_s(e)
    }
    
    function dirkey(path,depth,  n,i,k){
      n=split(path,a,"/")
      if (depth<=0||depth>n) depth=n
      k=a[1]
      for(i=2;i<=depth;i++) k=k"/"a[i]
      return k
    }
    
    function keep(l,m,w) {
      if (minl!="" && l+0<minl+0) return 0
      if (maxl!="" && l+0>maxl+0) return 0
      if (minc!="" && m+0<minc+0) return 0
      if (maxc!="" && m+0>maxc+0) return 0
      if (words && minw!="" && w+0<minw+0) return 0
      if (words && maxw!="" && w+0>maxw+0) return 0
      return 1
    }
    
    {
      if (words) {
        l=$1; m=$2; w=$3; p=$4
      } else {
        l=$1; m=$2; w=""; p=$3
      }
      
      if (!keep(l,m,w)) next
      
      cnt++
      lines[cnt]=l
      chars[cnt]=m
      wordsv[cnt]=w
      path[cnt]=p
      ext[cnt]=extname(p)
      dkey[cnt]=dirkey(p,dir_depth)
      
      sum_l+=l
      sum_m+=m
      if (words) sum_w+=w
      
      if (by_mode=="ext") {
        by_l[ext[cnt]]+=l
        by_m[ext[cnt]]+=m
        by_c[ext[cnt]]++
      }
      
      if (by_mode=="dir") {
        bydl[dkey[cnt]]+=l
        bydm[dkey[cnt]]+=m
        bydcount[dkey[cnt]]++
      }
    }
    
    END {
      if (fmt=="json") {
        for (i=1;i<=cnt;i++) {
          if (words) print lines[i]"\t"chars[i]"\t"wordsv[i]"\t"path[i]
          else print lines[i]"\t"chars[i]"\t"path[i]
        }
        printf "\n--SUMMARY--\n"
        if (words) printf "%d\t%d\t%d\tTOTAL\n", sum_l, sum_m, sum_w
        else printf "%d\t%d\tTOTAL\n", sum_l, sum_m
        printf "--FILESCOUNT--\n%d\n", cnt
        if (by_mode=="ext") {
          for (k in by_l) printf "BYEXT\t%s\t%d\t%d\t%d\n", k, by_l[k], by_m[k], by_c[k]
        }
        if (by_mode=="dir") {
          for (k in bydl) printf "BYDIR\t%s\t%d\t%d\t%d\n", k, bydl[k], bydm[k], bydcount[k]
        }
        exit
      }
      
      # ソート処理
      for (i=1;i<=cnt;i++) {
        key[i]=(sort_key=="lines"?lines[i]:(sort_key=="chars"?chars[i]:(sort_key=="words"?wordsv[i]:(sort_key=="name"?path[i]:(sort_key=="ext"?ext[i]:lines[i])))))
      }
      for (i=1;i<=cnt;i++) idx[i]=i
      for (i=1;i<=cnt;i++){
        sel=i
        for(j=i+1;j<=cnt;j++){
          if (sort_desc){
            if ((sort_key=="lines"||sort_key=="chars"||sort_key=="words")?(key[idx[j]]+0>key[idx[sel]]+0):(key[idx[j]]>key[idx[sel]])) sel=j
          } else {
            if ((sort_key=="lines"||sort_key=="chars"||sort_key=="words")?(key[idx[j]]+0<key[idx[sel]]+0):(key[idx[j]]<key[idx[sel]])) sel=j
          }
        }
        t=idx[i]; idx[i]=idx[sel]; idx[sel]=t
      }

      # ファイル一覧出力
      if (!total_only && !summary_only) {
        if (fmt=="table") {
          print ""
          if (words) { print "    LINES\t CHARACTERS\t   WORDS\tFILE" }
          else { print "    LINES\t CHARACTERS\tFILE" }
          print "----------------------------------------------"
        } else if (fmt=="csv") {
          if (words) print "lines,chars,words,file"
          else print "lines,chars,file"
        } else if (fmt=="tsv") {
          if (words) print "lines\tchars\twords\tfile"
          else print "lines\tchars\tfile"
        }
        
        limit=(top>0 && top<cnt)?top:cnt
        for(n=1;n<=limit;n++){
          i=idx[n]
          if (fmt=="table"){
            if (words) printf "%10d\t%10d\t%7d\t%s\n", lines[i], chars[i], wordsv[i], path[i]
            else printf "%10d\t%10d\t%s\n", lines[i], chars[i], path[i]
          } else if (fmt=="csv"){
            file=path[i]
            gsub("\"","\"\"",file)
            if (words) printf "%d,%d,%d,\"%s\"\n", lines[i], chars[i], wordsv[i], file
            else printf "%d,%d,\"%s\"\n", lines[i], chars[i], file
          } else if (fmt=="tsv"){
            if (words) printf "%d\t%d\t%d\t%s\n", lines[i], chars[i], wordsv[i], path[i]
            else printf "%d\t%d\t%s\n", lines[i], chars[i], path[i]
          }
        }
        if (fmt=="table") print "---"
      }

      # 拡張子別サマリ
      if (!total_only && by_mode=="ext") {
        if (fmt=="table") {
          print "[By Extension]"
          printf "%10s\t%10s\t%s\n","LINES","CHARACTERS","EXT"
        } else if (fmt=="csv") {
          print "ext_lines,ext_chars,ext,count"
        } else if (fmt=="tsv") {
          print "ext_lines\text_chars\text\tcount"
        }
        
        for(k in by_l){
          if (fmt=="table"){
            printf "%10d\t%10d\t%s (%d files)\n", by_l[k], by_m[k], (k==""?"(noext)":k), by_c[k]
          } else if (fmt=="csv"){
            ek=(k==""?"(noext)":k)
            gsub("\"","\"\"",ek)
            printf "%d,%d,\"%s\",%d\n", by_l[k], by_m[k], ek, by_c[k]
          } else if (fmt=="tsv"){
            printf "%d\t%d\t%s\t%d\n", by_l[k], by_m[k], (k==""?"(noext)":k), by_c[k]
          }
        }
        if (fmt=="table") print "---"
      }

      # ディレクトリ別サマリ
      if (!total_only && by_mode=="dir") {
        if (fmt=="table") {
          print "[By Directory]"
          printf "%10s\t%10s\t%s\n","LINES","CHARACTERS","DIR (depth="dir_depth")"
        } else if (fmt=="csv") {
          print "dir_lines,dir_chars,dir,count"
        } else if (fmt=="tsv") {
          print "dir_lines\tdir_chars\tdir\tcount"
        }
        
        for(k in bydl){
          if (fmt=="table"){
            printf "%10d\t%10d\t%s (%d files)\n", bydl[k], bydm[k], k, bydcount[k]
          } else if (fmt=="csv"){
            dk=k
            gsub("\"","\"\"",dk)
            printf "%d,%d,\"%s\",%d\n", bydl[k], bydm[k], dk, bydcount[k]
          } else if (fmt=="tsv"){
            printf "%d\t%d\t%s\t%d\n", bydl[k], bydm[k], k, bydcount[k]
          }
        }
        if (fmt=="table") print "---"
      }

      # 合計
      if (fmt=="table") {
        if (words) printf "%10d\t%10d\t%7d\tTOTAL (%d files)\n\n", sum_l, sum_m, sum_w, cnt
        else printf "%10d\t%10d\tTOTAL (%d files)\n\n", sum_l, sum_m, cnt
      } else if (fmt=="csv") {
        if (words) printf "%d,%d,%d,\"TOTAL\"\n", sum_l, sum_m, sum_w
        else printf "%d,%d,\"TOTAL\"\n", sum_l, sum_m
      } else if (fmt=="tsv") {
        if (words) printf "%d\t%d\t%d\tTOTAL\n", sum_l, sum_m, sum_w
        else printf "%d\t%d\tTOTAL\n", sum_l, sum_m
      }
    }
  '
  awk "$AWK_PROG"
}

# ========================================
# JSON生成関数
# ========================================
make_json() {
  python3 - <<'PY'
import json,sys
rows=[]; byext=[]; bydir=[]
sum_l=sum_c=sum_w=0
files_count=0
section='files'
words_mode=False

for raw in sys.stdin:
    raw=raw.rstrip('\n')
    if raw=='--SUMMARY--':
        section='summary'
        continue
    if raw=='--FILESCOUNT--':
        section='filescount'
        continue
    if raw.startswith('BYEXT\t'):
        _,ext,l,c,cnt = raw.split('\t')
        byext.append({"ext":ext or "", "lines":int(l), "chars":int(c), "count":int(cnt)})
        continue
    if raw.startswith('BYDIR\t'):
        _,dk,l,c,cnt = raw.split('\t')
        bydir.append({"dir":dk, "lines":int(l), "chars":int(c), "count":int(cnt)})
        continue
    
    if section=='files' and raw:
        parts = raw.split('\t')
        if len(parts)==4:
            l,c,w,p = parts
            words_mode=True
            rows.append({"file":p, "lines":int(l), "chars":int(c), "words":int(w)})
        elif len(parts)>=3:
            l,c,p = parts[0],parts[1],parts[2]
            rows.append({"file":p, "lines":int(l), "chars":int(c)})
    elif section=='summary' and raw:
        parts=raw.split('\t')
        if len(parts)>=3 and parts[0].isdigit() and parts[1].isdigit():
            sum_l=int(parts[0])
            sum_c=int(parts[1])
            if len(parts)>=4 and parts[2].isdigit():
                sum_w=int(parts[2])
    elif section=='filescount' and raw:
        try:
            files_count=int(raw.strip())
        except:
            pass

if files_count==0:
    files_count=len(rows)

summary={"lines":sum_l, "chars":sum_c, "files":files_count}
if sum_w:
    summary["words"]=sum_w

out={"files":rows, "summary":summary}
if byext:
    out["by_extension"]=byext
if bydir:
    out["by_directory"]=bydir

print(json.dumps(out, ensure_ascii=False, indent=2))
PY
}

# ========================================
# ヘルプ表示
# ========================================
usage() {
  cat <<USAGE
Usage: count_lines [OPTIONS] [PATH ...]

Options:
  --format table|csv|tsv|json
  --sort lines|chars|words|name|ext
  --desc                        降順
  --top N                       上位 N のみ表示
  --by ext|dir[=N]              サマリ軸: 拡張子 or ディレクトリ(先頭 N 階層)
  --summary-only                一覧を出さずサマリのみ
  --total-only                  合計のみ（一覧やサマリなし）

  --include 'PATTERN'           含めるファイル名（basename）glob（複数可）
  --exclude 'PATTERN'           除外するファイル名（basename）glob（複数可）
  --include-path 'PATTERN'      含めるパス（フルパス）glob（複数可）
  --exclude-path 'PATTERN'      除外するパス（フルパス）glob（複数可）
  --exclude-dir 'PATTERN'       除外ディレクトリ（グロブ、複数可・剪定）
  --ext 'py,js,ts'              拡張子フィルタ（カンマ区切り; 大小無視）
  --max-size SIZE               例: 50MiB, 200M, +10M（find 準拠・未指定は上限）
  --min-lines N / --max-lines N
  --min-chars N / --max-chars N
  --words                       単語数も計測・出力（wc -w）
  --min-words N / --max-words N
  --text-only                   バイナリらしいファイルを除外（grep -Iq）
  --files-from FILE             入力ファイル一覧（改行区切り）
  --files-from0 FILE            入力ファイル一覧（NUL 区切り）

  --hidden                      隠しファイル/ディレクトリも対象
  --follow                      シンボリックリンクを辿る
  --git                         .gitignore を尊重して列挙
  --jobs N                      並列数（既定: CPU コア数）
  --no-default-prune            既定の剪定(.git,node_modules 等)を無効化
  --abs-path                    出力パスを絶対パスにする
  --trim-root PATH              出力時に PATH を先頭から取り除く
  --mtime-since DATETIME        この日時以降に更新されたファイルのみ
  --mtime-until DATETIME        この日時以前に更新されたファイルのみ
  --no-color                    色なし（table 出力時）
  --version                     バージョン表示
  -h, --help                    このヘルプ

Examples:
  count_lines --by ext --sort chars --desc --top 20
  count_lines --git --ext 'py,js' --format json > stats.json
  count_lines --by dir=2 --summary-only src tests
  count_lines --words --sort words --desc --min-words 10
  count_lines --mtime-since '2025-01-01' --mtime-until '2025-10-01'
USAGE
}

# ========================================
# 引数パース
# ========================================
parse_arguments() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --format) FORMAT="$2"; shift 2;;
      --sort) SORT_KEY="$2"; shift 2;;
      --desc) SORT_DESC=1; shift;;
      --top) TOP_N="$2"; shift 2;;
      --by)
        case "$2" in
          ext) BY_MODE="ext";;
          dir|dir=*) BY_MODE="dir"; DIR_DEPTH="${2#dir=}"; [[ "$DIR_DEPTH" == "dir" ]] && DIR_DEPTH=1;;
          *) err "Unknown --by: $2"; exit 1;;
        esac
        shift 2;;
      --summary-only) SUMMARY_ONLY=1; shift;;
      --total-only) TOTAL_ONLY=1; shift;;
      --include) INCLUDE_PATTERNS+=("$2"); shift 2;;
      --exclude) EXCLUDE_PATTERNS+=("$2"); shift 2;;
      --include-path) INCLUDE_PATHS+=("$2"); shift 2;;
      --exclude-path) EXCLUDE_PATHS+=("$2"); shift 2;;
      --exclude-dir) EXCLUDE_DIRS+=("$2"); shift 2;;
      --ext) IFS=',' read -r -a EXT_FILTERS <<<"$2"; shift 2;;
      --max-size) MAX_SIZE="$2"; shift 2;;
      --min-lines) MIN_LINES="$2"; shift 2;;
      --max-lines) MAX_LINES="$2"; shift 2;;
      --min-chars) MIN_CHARS="$2"; shift 2;;
      --max-chars) MAX_CHARS="$2"; shift 2;;
      --words) WORDS=1; shift;;
      --min-words) MIN_WORDS="$2"; shift 2;;
      --max-words) MAX_WORDS="$2"; shift 2;;
      --text-only) TEXT_ONLY=1; shift;;
      --files-from) FILES_FROM="$2"; shift 2;;
      --files-from0) FILES_FROM0="$2"; shift 2;;
      --hidden) HIDDEN=1; shift;;
      --follow) FOLLOW=1; shift;;
      --git) USE_GIT=1; shift;;
      --jobs) JOBS="$2"; shift 2;;
      --no-color) NO_COLOR=1; shift;;
      --no-default-prune) DEFAULT_PRUNE=0; shift;;
      --abs-path) ABS_PATH=1; shift;;
      --trim-root) TRIM_ROOT="$2"; shift 2;;
      --mtime-since) MTIME_SINCE="$2"; shift 2;;
      --mtime-until) MTIME_UNTIL="$2"; shift 2;;
      --version) echo "count_lines v$VERSION"; exit 0;;
      -h|--help) usage; exit 0;;
      --) shift; break;;
      -*) err "Unknown option: $1"; usage; exit 1;;
      *) PATHS+=("$1"); shift;;
    esac
  done
  [[ $# -gt 0 ]] && PATHS+=("$@")
  [[ -z "$FILES_FROM" && -z "$FILES_FROM0" && ${#PATHS[@]} -eq 0 ]] && PATHS+=(".")
}

# ========================================
# 依存関係チェック
# ========================================
validate_dependencies() {
  if [[ "$FORMAT" == json ]]; then
    command -v python3 >/dev/null 2>&1 || err "--format json requires python3"
  fi
  if [[ $USE_GIT -eq 1 ]]; then
    command -v git >/dev/null 2>&1 || err "--git requires git"
  fi
  if [[ $TEXT_ONLY -eq 1 ]]; then
    command -v grep >/dev/null 2>&1 || err "--text-only requires grep"
  fi
}

# ========================================
# メイン処理
# ========================================
main() {
  parse_arguments "$@"
  validate_dependencies
  
  JOBS=${JOBS:-$(cpu_cores)}
  
  if [[ "$FORMAT" == table ]] && is_tty; then
    info "count_lines v$VERSION · parallel=$JOBS"
  fi
  
  if [[ "$FORMAT" == json ]]; then
    list_files | measure_wc | postprocess | make_json
  else
    list_files | measure_wc | postprocess
  fi
}

main "$@"
EOF
}

# ========================================
# インストール処理
# ========================================
get_install_target() {
  if [[ "$INSTALL_MODE" == "global" ]]; then
    echo "/usr/local/bin/count_lines"
  else
    echo "$HOME/.local/bin/count_lines"
  fi
}

install_user() {
  local prefix="$HOME/.local/bin"
  local target="$prefix/count_lines"
  
  mkdir -p "$prefix"
  
  if [[ "$DRYRUN" == yes ]]; then
    emit_count_lines
    return
  fi
  
  emit_count_lines > "$target"
  chmod +x "$target"
  info "Installed: $target"
  
  setup_shell_path
  add_to_current_path
}

install_global() {
  local prefix="/usr/local/bin"
  local target="$prefix/count_lines"
  
  sudo mkdir -p "$prefix"
  
  if [[ "$DRYRUN" == yes ]]; then
    emit_count_lines
    return
  fi
  
  emit_count_lines | sudo tee "$target" >/dev/null
  sudo chmod +x "$target"
  info "Installed (global): $target"
}

uninstall() {
  local target
  target=$(get_install_target)
  
  if [[ -e "$target" ]]; then
    if [[ "$INSTALL_MODE" == "global" ]]; then
      sudo rm -f "$target"
    else
      rm -f "$target"
    fi
    info "Uninstalled: $target"
  else
    info "Not found: $target"
  fi
}

# ========================================
# 引数パース（インストーラー）
# ========================================
parse_installer_args() {
  for arg in "$@"; do
    case "$arg" in
      --global) INSTALL_MODE="global" ;;
      --uninstall) UNINSTALL="yes" ;;
      --dry-run) DRYRUN="yes" ;;
      *) err "Unknown option: $arg" ;;
    esac
  done
}

# ========================================
# メイン処理（インストーラー）
# ========================================
main() {
  parse_installer_args "$@"
  check_required_commands
  validate_options
  
  if [[ "$UNINSTALL" == "yes" ]]; then
    uninstall
    exit 0
  fi
  
  if [[ "$INSTALL_MODE" == "global" ]]; then
    install_global
  else
    install_user
  fi
  
  echo
  echo "Done. Try:  count_lines --help"
  echo "Which:     $(command -v count_lines || echo 'not in PATH yet')"
}

main "$@"