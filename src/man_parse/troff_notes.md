### Line breaks & vertical space
To summarize, the following macros cause a line break with the insertion of vertical space (which amount can be changed with the PD macro): SH, SS, TP, LP (PP, P), IP, and HP.
The macros RS and RE also cause a break but do not insert vertical space.
Finally, the macros SH, SS, LP (PP, P), and RS reset the indentation to its default value. 
https://www.gnu.org/software/groff/manual/html_node/Man-usage.html


### .RS
.RS [Indent]

Increases relative indent (initially zero). Indent all output an extra number of units from the left margin as specified by the Indent variable.

If the Indent variable is omitted, the previous Indent value is used. This value is set to its default (5 ens for the nroff command and 7.2 ens for the troff command) by the .TH format macro, .P format macro, and .RS format macro, and restored by the .RE format macro. The default unit for Indent is ens.

### .TP
.TP [Indent]

Begins indented paragraph with hanging tag. The next input line that contains text is the tag. If the tag does not fit, it is printed on a separate line.

If the Indent variable is omitted, the previous Indent value is used. This value is set to its default (5 ens for the nroff command and 7.2 ens for the troff command) by the .TH format macro, .P format macro, and .RS format macro, and restored by the .RE format macro. The default unit for Indent is ens.

### .IP
.IP [tag [indent]]

Indents paragraph, with hanging tag X. Y specifies spaces to indent.

Initial Value: -

Break: yes

Reset: yes 