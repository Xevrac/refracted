using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ShowScriptedDisplay
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ShowScriptedDisplay); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ShowScriptedDisplay)obj;
            //  Serialize ScriptedDisplayType
            s.Write(value.ScriptedDisplayType);
            //  Serialize XCoordinate
            s.Write(value.XCoordinate);
            //  Serialize YCoordinate
            s.Write(value.YCoordinate);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ShowScriptedDisplay)) as Rts.CnC.Messages.Client.ShowScriptedDisplay;
            //  Deserialize ScriptedDisplayType
            s.Read(out value.ScriptedDisplayType);
            //  Deserialize XCoordinate
            s.Read(out value.XCoordinate);
            //  Deserialize YCoordinate
            s.Read(out value.YCoordinate);

            return value;
        }
        
    }
}
