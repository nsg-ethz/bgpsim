.PHONY: all clean

L=logo

all     : light_only.svg light_text.svg dark_only.svg dark_text.svg blue_only.svg blue_text.svg favicon_only.svg

clean   :
	@$(RM) *.log *.aux *.pdf *.svg

%.svg   : $L.tex
	@# $@            --> $C_$M.svg
	@# (basename $@) --> $C_$M
	@# (subst ...)   --> $C{1} \def\$M
	@# combine       --> \def\$C{1} \def\$M{1}
	@# command       --> pdflatex `\def\$C{1} \def\$M{1} \input{$L.tex}' > /dev/null
	pdflatex '\def\$(subst _,{1} \def\,$(basename $@)){1} \input{$L.tex}'
	@mv $L.pdf $(basename $@).pdf
	inkscape --pdf-poppler --export-text-to-path --export-type=svg --export-filename=$@ $(basename $@).pdf
