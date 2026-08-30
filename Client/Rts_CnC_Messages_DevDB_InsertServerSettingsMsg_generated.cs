using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_DevDB_InsertServerSettingsMsg
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.DevDB.InsertServerSettingsMsg); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.DevDB.InsertServerSettingsMsg)obj;
            //  Serialize ConfigurationName
            s.Write(value.ConfigurationName);
            //  Serialize ServerType
            s.Write(value.ServerType);
            //  Serialize Settings
            s.Write(value.Settings);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.DevDB.InsertServerSettingsMsg)) as Rts.CnC.Messages.DevDB.InsertServerSettingsMsg;
            //  Deserialize ConfigurationName
            s.Read(out value.ConfigurationName);
            //  Deserialize ServerType
            s.Read(out value.ServerType);
            //  Deserialize Settings
            s.Read(out value.Settings);

            return value;
        }
        
    }
}
