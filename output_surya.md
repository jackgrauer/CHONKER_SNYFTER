DocLayNet: A Large Human-Annotated Dataset for
<b>Document-Layout Analysis</b>
Birgit Pfitzmann
Michele Dolfi
Christoph Auer
<b>IBM</b> Research
<b>IBM</b> Research
<b>IBM</b> Research
Rueschlikon, Switzerland
Rueschlikon, Switzerland
Rueschlikon, Switzerland
bpf@zurich.ibm.com
cau@zurich.ibm.com
dol@zurich.ibm.com
Ahmed S. Nassar
Peter Staar
<b>IBM</b> Research
<b>IBM</b> Research
Rueschlikon, Switzerland
Rueschlikon, Switzerland
arXiv:2206.01062v1 [cs.CV] 2 Jun 2022
taa@zurich.ibm.com
ahn@zurich.ibm.com
<b>ABSTRACT</b>
Accurate document layout analysis is a key requirement for high-
quality PDF document conversion. With the recent availability of
public, large ground-truth datasets such as PubLayNet and DocBank,
deep-learning models have proven to be very effective at layout
detection and segmentation. While these datasets are of adequate
size to train such models, they severely lack in layout variability
since they are sourced from scientific article repositories such as
PubMed and arXiv only. Consequently, the accuracy of the layout
<b>OBARSKA</b>
segmentation drops significantly when these models are applied
<b>OBARSKA</b>
ш
on more challenging and diverse layouts. In this paper, we present
DocLayNet, a new, publicly available, document-layout annotation
dataset in COCO format. It contains 80863 manually annotated
pages from diverse data sources to represent a wide variability in
layouts. For each PDF page, the layout annotations provide labelled
bounding-boxes with a choice of 11 distinct classes. DocLayNet
Looking back on
also provides a subset of double- and triple-annotated pages to
175 years of
determine the inter-annotator agreement. In multiple experiments,
looking forward.
we provide baseline accuracy scores (in mAP) for a set of popular
object detection models. We also demonstrate that these models
fall approximately 10% behind the inter-annotator agreement. Fur-
thermore, we provide evidence that DocLayNet is of sufficient size.
Lastly, we compare models trained on PubLayNet, DocBank and
DocLayNet, showing that layout predictions of the DocLayNet-
trained models are more robust and thus the preferred choice for
general-purpose document-layout analysis.
Figure 1: Four examples of complex page layouts across dif-
<b>CCS CONCEPTS</b>
ferent document categories
<math>\bullet</math> Information systems <math>\to</math> Document structure; <math>\bullet</math> Applied com-
puting <math>\rightarrow</math> Document analysis; <math>\cdot</math> Computing methodologies
<math>\rightarrow</math> Machine learning; Computer vision; Object detection;
<b>KEYWORDS</b>
PDF document conversion, layout segmentation, object-detection,
data set, Machine Learning
Permission to make digital or hard copies of part or all of this work for personal or
<b>ACM Reference Format:</b>
classroom use is granted without fee provided that copies are not made or distributed
for profit or commercial advantage and that copies bear this notice and the full citation
Birgit Pfitzmann, Christoph Auer, Michele Dolfi, Ahmed S. Nassar, and Peter
on the first page. Copyrights for third-party components of this work must be honored.
Staar. 2022. DocLayNet: A Large Human-Annotated Dataset for Document-
For all other uses, contact the owner/author(s).
Layout Analysis. In Proceedings of the 28th ACM SIGKDD Conference on
KDD '22, August 14-18, 2022, Washington, DC, USA
Knowledge Discovery and Data Mining (KDD '22), August 14-18, 2022, Wash-
© 2022 Copyright held by the owner/author(s).
ington, DC, USA. ACM, New York, NY, USA, 9 pages. https://doi.org/10.1145/
ACM ISBN 978-1-4503-9385-0/22/08.
https://doi.org/10.1145/3534678.3539043
3534678.3539043