using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_GameWidgetStatusMessage_Debug
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.GameWidgetStatusMessage_Debug); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.GameWidgetStatusMessage_Debug)obj;
            //  Serialize DebugText
            s.Write(value.DebugText);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.GameWidgetStatusMessage_Debug)) as Rts.CnC.Messages.Client.GameWidgetStatusMessage_Debug;
            //  Deserialize DebugText
            s.Read(out value.DebugText);

            return value;
        }
        
    }
}
