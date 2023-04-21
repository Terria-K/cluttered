namespace Cluttered;

public static class AtlasPath 
{
  {{#each atlas.frames as frames}}
  public const string {{ replace (replace @key "/" "_") "\\" "_" }} = "{{@key}}";
  {{/each}}
}