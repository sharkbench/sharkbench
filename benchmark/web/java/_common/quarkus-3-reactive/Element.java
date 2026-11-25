package org.acme;

/** One entry of the data source's element.json, bound by Jackson (primitive ints, no boxing). */
public class Element {
    public String name;
    public int number;
    public int group;
}
