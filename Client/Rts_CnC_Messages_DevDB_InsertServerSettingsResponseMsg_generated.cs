using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_DevDB_InsertServerSettingsResponseMsg
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.DevDB.InsertServerSettingsResponseMsg); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.DevDB.InsertServerSettingsResponseMsg)obj;

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.DevDB.InsertServerSettingsResponseMsg)) as Rts.CnC.Messages.DevDB.InsertServerSettingsResponseMsg;

            return value;
        }
        
    }
}
