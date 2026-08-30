using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_GameWidgetStatusMessage
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.GameWidgetStatusMessage); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.GameWidgetStatusMessage)obj;
            //  Serialize TextId
            s.Write(value.TextId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.GameWidgetStatusMessage)) as Rts.CnC.Messages.Client.GameWidgetStatusMessage;
            //  Deserialize TextId
            s.Read(out value.TextId);

            return value;
        }
        
    }
}
