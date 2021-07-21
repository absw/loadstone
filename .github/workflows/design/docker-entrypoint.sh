#!/usr/bin/env bash

in_path="${1}"
printf "Got input path: %s\n" "${in_path}"
out_path="${2}"
printf "Got output path: %s\n" "${out_path}"
tmp_doc_path="${in_path}_tmp"
work_dir=`pwd`

function main {
    rm -rf "${out_path}"
    mkdir -p "${out_path}"

    generate_pdf_files
    generate_html_files
}

function generate_pdf_files {
    printf "Generating pdf documents...\n"
    cd "${work_dir}"
    rm -rf "${tmp_doc_path}"
    mkdir -p "${tmp_doc_path}"
    cp -r "${in_path}" "${tmp_doc_path}"

    cd "${tmp_doc_path}/${in_path}"
    for in_file in `ls *.md`; do
        if [[ -f "${in_file}" ]]; then
            out_file=`echo "${in_file%.*}"`

            #Fix the links to any additional documents
            sed -i 's/\.md/\.pdf/g' "${in_file}"

            printf "Attempting compile of: %s to %s\n" "${in_file}" "${out_path}/${out_file}.pdf"
            pandoc ${in_file} -s -o "${work_dir}/${out_path}/${out_file}.pdf"
        fi
    done
    printf "\n"
}

function generate_html_files {
    printf "Generating html documents...\n"
    cd "${work_dir}"
    rm -rf "${tmp_doc_path}"
    mkdir -p "${tmp_doc_path}"
    cp -r "${in_path}" "${tmp_doc_path}"

    cd "${tmp_doc_path}/${in_path}"
    for in_file in `ls *.md`; do
        if [[ -f "${in_file}" ]]; then
            out_file=`echo "${in_file%.*}"`

            #Fix the links to any additional documents
            sed -i 's/\.md/\.html/g' "${in_file}"

            printf "Attempting compile of: %s to %s\n" "${in_file}" "${out_path}/${out_file}.html"
            pandoc ${in_file} -s -o "${work_dir}/${out_path}/${out_file}.html" --metadata pagetitle="${out_file}"
        fi
    done
    printf "\n"
}

function copy_images {
    cd "${work_dir}"
    if [[ -d "${in_path}/images" ]]; then
        cp -r "${in_path}/images" ${out_path}
    fi
}

main "$@"

copy_images

exit
